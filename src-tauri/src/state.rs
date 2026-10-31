use serde::Serialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Serialize, Clone)]
pub struct ModeInfo {
    pub res: String,
    pub scaling: bool,
}

#[derive(Serialize, Clone, Default)]
pub struct DisplayInfo {
    pub ctx_id: String,
    pub persistent_id: String,
    pub serial_id: String,
    pub type_str: String,
    pub res: String,
    pub hz: Option<u32>,
    pub color_depth: Option<u8>,
    pub scaling: bool,
    pub origin: (i32, i32),
    pub degree: u32,
    pub enabled: bool,
    pub modes: Vec<ModeInfo>,
    // 系统主显示器（菜单栏所在，Origin 行带 "- main display"）
    pub main: bool,
    // 镜像从屏：所在镜像组主屏的 serial id；非镜像从屏为 None
    pub mirror_of: Option<String>,
    // 面板物理尺寸（对角线英寸），取自 EDID 物理尺寸；取不到为 None
    pub size_in: Option<u32>,
    // 型号名（U2790B / Color LCD …），来自 system_profiler；取不到为 None
    pub model: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct AppState {
    pub displays: Vec<DisplayInfo>,
}

fn run_displayplacer(program: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("无法运行 displayplacer：{e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn parse_kv(line: &str) -> Option<(&str, &str)> {
    let (k, v) = line.split_once(':')?;
    Some((k.trim(), v.trim()))
}

fn parse_origin(v: &str) -> (i32, i32) {
    let Some(open) = v.find('(') else {
        return (0, 0);
    };
    let Some(close) = v[open..].find(')') else {
        return (0, 0);
    };
    let inner = &v[open + 1..open + close];
    match inner.split_once(',') {
        Some((x, y)) => (x.trim().parse().unwrap_or(0), y.trim().parse().unwrap_or(0)),
        None => (0, 0),
    }
}

fn mode_width(res: &str) -> i32 {
    res.split_once('x')
        .and_then(|(w, _)| w.parse().ok())
        .unwrap_or(0)
}

fn aspect_diff(res: &str, native_ratio: f64) -> f64 {
    let Some((w, h)) = res.split_once('x') else {
        return 1.0;
    };
    let (Ok(w), Ok(h)) = (w.parse::<f64>(), h.parse::<f64>()) else {
        return 1.0;
    };
    ((w / h) - native_ratio).abs() / native_ratio
}

// 按 macOS 系统设置的逻辑过滤：只保留与原生宽高比一致、且是 HiDPI(scaling:on)
// 或原生分辨率的模式；同一分辨率优先取 scaling:on 的变体。
fn filter_modes(raw: &[(String, bool)]) -> Vec<ModeInfo> {
    let native_width = raw.iter().map(|(r, _)| mode_width(r)).max().unwrap_or(0);
    if native_width == 0 {
        return Vec::new();
    }
    let native_ratio = raw
        .iter()
        .filter(|(r, _)| mode_width(r) == native_width)
        .max_by_key(|(_, s)| *s)
        .and_then(|(r, _)| r.split_once('x'))
        .and_then(|(w, h)| w.parse::<f64>().ok().zip(h.parse::<f64>().ok()))
        .map(|(w, h)| w / h)
        .unwrap_or(1.7778);

    let mut best: HashMap<String, bool> = HashMap::new();
    for (res, scaling) in raw {
        let width = mode_width(res);
        if aspect_diff(res, native_ratio) > 0.02 {
            continue;
        }
        if !scaling && width != native_width {
            continue;
        }
        best.entry(res.clone())
            .and_modify(|s| *s = *s || *scaling)
            .or_insert(*scaling);
    }

    let mut modes: Vec<ModeInfo> = best
        .into_iter()
        .map(|(res, scaling)| ModeInfo { res, scaling })
        .collect();
    modes.sort_by_key(|m| std::cmp::Reverse(mode_width(&m.res)));
    modes
}

pub fn parse_output(out: &str) -> Result<AppState, String> {
    let mut displays: Vec<DisplayInfo> = Vec::new();
    let mut current: Option<DisplayInfo> = None;
    let mut raw_modes: Vec<(String, bool)> = Vec::new();
    let mut in_modes = false;

    for line in out.lines() {
        let line = line.trim_end();
        if let Some(v) = line.strip_prefix("Persistent screen id:") {
            if let Some(mut d) = current.take() {
                d.modes = filter_modes(&raw_modes);
                raw_modes.clear();
                displays.push(d);
            }
            let mut d = DisplayInfo {
                enabled: true,
                ..Default::default()
            };
            d.persistent_id = v.trim().to_string();
            current = Some(d);
            in_modes = false;
            continue;
        }
        let Some(d) = current.as_mut() else { continue };
        if in_modes {
            if let Some(mode_part) = line.trim().strip_prefix("mode ") {
                let mut m_res = String::new();
                let mut m_scaling = false;
                for tok in mode_part.split_whitespace() {
                    if let Some(v) = tok.strip_prefix("res:") {
                        m_res = v.to_string();
                    } else if let Some(v) = tok.strip_prefix("scaling:") {
                        m_scaling = v == "on";
                    }
                }
                if !m_res.is_empty() {
                    raw_modes.push((m_res, m_scaling));
                }
                continue;
            }
            in_modes = false;
        }
        if line.starts_with("Resolutions for rotation") {
            in_modes = true;
            continue;
        }
        let Some((k, v)) = parse_kv(line) else {
            continue;
        };
        match k {
            "Contextual screen id" => d.ctx_id = v.to_string(),
            "Serial screen id" => d.serial_id = v.to_string(),
            "Type" => d.type_str = v.to_string(),
            "Resolution" => d.res = v.to_string(),
            "Hertz" => d.hz = v.parse().ok(),
            "Color Depth" => d.color_depth = v.parse().ok(),
            "Scaling" => d.scaling = v == "on",
            "Origin" => {
                d.origin = parse_origin(v);
                d.main = v.contains("main display");
            }
            "Rotation" => {
                d.degree = v
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0);
            }
            "Enabled" => d.enabled = v == "true",
            _ => {}
        }
    }
    if let Some(mut d) = current.take() {
        d.modes = filter_modes(&raw_modes);
        displays.push(d);
    }

    if displays.is_empty() {
        return Err("displayplacer 未返回任何显示器".to_string());
    }

    // 镜像组解析：末尾建议命令的 id token 形如 id:主+从1+从2，可同时存在多组
    if let Some(cmd_line) = out
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with("displayplacer "))
    {
        let resolve = |id: &str| -> Option<String> {
            displays
                .iter()
                .find(|d| d.persistent_id == id || d.ctx_id == id || d.serial_id == id)
                .map(|d| d.serial_id.clone())
        };
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        for tok in cmd_line.split_whitespace() {
            let Some(v) = tok.trim_matches('"').strip_prefix("id:") else {
                continue;
            };
            let parts: Vec<&str> = v.split('+').collect();
            if parts.len() < 2 {
                continue;
            }
            let Some(primary) = resolve(parts[0]) else {
                continue;
            };
            let followers = parts[1..].iter().filter_map(|p| resolve(p)).collect();
            groups.push((primary, followers));
        }
        for (primary, followers) in groups {
            for follower in followers {
                if let Some(d) = displays.iter_mut().find(|d| d.serial_id == follower) {
                    d.mirror_of = Some(primary.clone());
                }
            }
        }
    }

    Ok(AppState { displays })
}

pub fn read_state(program: &str) -> Result<AppState, String> {
    let out = run_displayplacer(program, &["list"])?;
    let mut st = parse_output(&out)?;
    let pairs: Vec<(String, String)> = st
        .displays
        .iter()
        .map(|d| (d.ctx_id.clone(), d.serial_id.clone()))
        .collect();
    let extra = crate::sysinfo::extras(&pairs);
    // 内建屏的 EDID 不暴露物理尺寸；CGDisplayScreenSize 只在未参与镜像时可信
    let mirrored: Vec<bool> = st
        .displays
        .iter()
        .map(|d| {
            d.mirror_of.is_some()
                || st
                    .displays
                    .iter()
                    .any(|x| x.mirror_of.as_deref() == Some(d.serial_id.as_str()))
        })
        .collect();
    for (d, is_mirrored) in st.displays.iter_mut().zip(mirrored) {
        if let Some(e) = extra.get(&d.ctx_id) {
            d.size_in = e.size_in;
            d.model = e.model.clone();
        }
        if d.size_in.is_none() && !is_mirrored {
            d.size_in = cg_inches(&d.ctx_id);
        }
    }
    Ok(st)
}

#[cfg(target_os = "macos")]
fn cg_inches(ctx_id: &str) -> Option<u32> {
    use core_graphics::display::CGDisplayScreenSize;
    let id: u32 = ctx_id.parse().ok()?;
    let mm = unsafe { CGDisplayScreenSize(id) };
    if mm.width <= 0.0 || mm.height <= 0.0 {
        return None;
    }
    Some(((mm.width * mm.width + mm.height * mm.height).sqrt() / 25.4).round() as u32)
}

#[cfg(not(target_os = "macos"))]
fn cg_inches(_ctx_id: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mirror_output() {
        let out = include_str!("../tests/fixtures/mirror.txt");
        let state = parse_output(out).expect("解析失败");
        assert_eq!(state.displays.len(), 2);
        let external = state
            .displays
            .iter()
            .find(|d| d.type_str == "27 inch external screen")
            .expect("应有外接屏");
        assert!(external.main, "镜像组主屏应标记为系统主显示器");
        assert_eq!(external.mirror_of, None, "组主屏不是镜像从屏");
        assert_eq!(external.res, "3008x1692");
        let builtin = state
            .displays
            .iter()
            .find(|d| d.type_str == "MacBook built in screen")
            .expect("应有内置屏");
        assert_eq!(
            builtin.mirror_of.as_deref(),
            Some(external.serial_id.as_str()),
            "内置屏应解析为镜像外接屏"
        );
        assert!(!builtin.main);
        assert_eq!(builtin.res, "3008x1692");
        for d in &state.displays {
            assert!(!d.modes.is_empty(), "{} 的模式列表不应为空", d.type_str);
            for m in &d.modes {
                assert!(m.res.contains('x'), "模式 {} 格式异常", m.res);
            }
        }
        let has_1512x982 = builtin.modes.iter().any(|m| m.res == "1512x982");
        assert!(has_1512x982, "内置屏应有 1512x982 缩放档");
        let no_16x9_in_builtin = builtin.modes.iter().all(|m| {
            let (w, h) = m.res.split_once('x').unwrap();
            let (w, h): (f64, f64) = (w.parse().unwrap(), h.parse().unwrap());
            (w / h - 3024.0 / 1964.0).abs() / (3024.0 / 1964.0) < 0.02
        });
        assert!(no_16x9_in_builtin, "内置屏不应混入 16:9 模式");
    }
}
