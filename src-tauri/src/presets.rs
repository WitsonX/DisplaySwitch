use serde::{Deserialize, Deserializer, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const PRESET_DIR: &str = "presets";

#[derive(Serialize, Clone)]
pub struct PresetEntry {
    pub name: String,
    pub config: ApplyConfig,
}

// v2 角色模型：role = main | extend | mirror；mirror 时 mirror_target 为目标屏 serial id
#[derive(Serialize, Deserialize, Clone)]
pub struct ApplyDisplay {
    pub ctx_id: String,
    pub role: String,
    pub mirror_target: Option<String>,
    pub res: String,
    pub scaling: bool,
    pub hz: Option<u32>,
    pub color_depth: Option<u8>,
    pub degree: u32,
    pub enabled: bool,
}

#[derive(Serialize, Clone)]
pub struct ApplyConfig {
    pub displays: Vec<ApplyDisplay>,
}

#[derive(Deserialize)]
struct ApplyConfigV2 {
    displays: Vec<ApplyDisplay>,
}

// v1 旧格式（mode + primary），读取时自动转换为角色模型
#[derive(Deserialize)]
struct LegacyDisplay {
    ctx_id: String,
    res: String,
    scaling: bool,
    degree: u32,
    hz: Option<u32>,
    color_depth: Option<u8>,
    enabled: bool,
}

#[derive(Deserialize)]
struct LegacyConfig {
    mode: String,
    primary: String,
    displays: Vec<LegacyDisplay>,
}

impl From<LegacyConfig> for ApplyConfig {
    fn from(legacy: LegacyConfig) -> Self {
        let displays = legacy
            .displays
            .into_iter()
            .map(|d| {
                let is_primary = d.ctx_id == legacy.primary;
                let (role, mirror_target) = if is_primary {
                    ("main".to_string(), None)
                } else if legacy.mode == "mirror" {
                    ("mirror".to_string(), Some(legacy.primary.clone()))
                } else {
                    ("extend".to_string(), None)
                };
                ApplyDisplay {
                    ctx_id: d.ctx_id,
                    role,
                    mirror_target,
                    res: d.res,
                    scaling: d.scaling,
                    hz: d.hz,
                    color_depth: d.color_depth,
                    degree: d.degree,
                    enabled: d.enabled,
                }
            })
            .collect();
        ApplyConfig { displays }
    }
}

impl<'de> Deserialize<'de> for ApplyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = serde_json::Value::deserialize(deserializer)?;
        if raw.get("mode").is_some() {
            let legacy =
                serde_json::from_value::<LegacyConfig>(raw).map_err(serde::de::Error::custom)?;
            Ok(ApplyConfig::from(legacy))
        } else {
            let v2 =
                serde_json::from_value::<ApplyConfigV2>(raw).map_err(serde::de::Error::custom)?;
            Ok(ApplyConfig {
                displays: v2.displays,
            })
        }
    }
}

fn presets_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法定位配置目录：{e}"))?
        .join(PRESET_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("无法创建预设目录：{e}"))?;
    Ok(dir)
}

fn sanitize_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .collect();
    let trimmed = cleaned.trim().to_string();
    if trimmed.is_empty() {
        "未命名".to_string()
    } else {
        trimmed
    }
}

pub fn save_preset(app: &AppHandle, name: &str, config: &ApplyConfig) -> Result<(), String> {
    let path = presets_dir(app)?.join(format!("{}.json", sanitize_name(name)));
    let json = serde_json::to_string_pretty(config).map_err(|e| format!("序列化失败：{e}"))?;
    fs::write(&path, json).map_err(|e| format!("写入预设失败：{e}"))
}

pub fn list_presets(app: &AppHandle) -> Result<Vec<PresetEntry>, String> {
    let dir = presets_dir(app)?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("读取预设目录失败：{e}"))? {
        let path = entry.map_err(|e| format!("读取预设目录失败：{e}"))?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("未命名")
            .to_string();
        let text = fs::read_to_string(&path).map_err(|e| format!("读取预设 {name} 失败：{e}"))?;
        let config: ApplyConfig =
            serde_json::from_str(&text).map_err(|e| format!("解析预设 {name} 失败：{e}"))?;
        out.push(PresetEntry { name, config });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn delete_preset(app: &AppHandle, name: &str) -> Result<(), String> {
    let path = presets_dir(app)?.join(format!("{}.json", sanitize_name(name)));
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("删除预设失败：{e}"))?;
    }
    Ok(())
}
