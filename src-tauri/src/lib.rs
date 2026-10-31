mod presets;
mod settings;
mod state;
mod sysinfo;

use presets::{ApplyConfig, ApplyDisplay};
#[cfg(target_os = "macos")]
use std::os::raw::c_void;
use std::process::Command;
#[cfg(target_os = "macos")]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::Emitter;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, LogicalSize, Manager, PhysicalPosition, WindowEvent,
};

const PANEL_WIDTH: f64 = 600.0;
const SINGLE_PANEL_WIDTH: f64 = 380.0;
const PANEL_HEIGHT: f64 = 520.0;

// 单屏时面板用窄窗口；计数由 get_state 刷新，托盘点击时据此选宽度
static LAST_DISPLAY_COUNT: AtomicUsize = AtomicUsize::new(2);

// 合盖/开盖/插拔会连续触发多次重配置回调，用序号防抖：800ms 内无新变化才通知前端
#[cfg(target_os = "macos")]
static DISPLAY_CHANGE_SEQ: AtomicU64 = AtomicU64::new(0);

#[cfg(target_os = "macos")]
unsafe extern "C" fn on_display_reconfigure(
    _display: core_graphics::display::CGDirectDisplayID,
    _flags: u32,
    user_info: *const c_void,
) {
    let app = &*(user_info as *const AppHandle);
    let seq = DISPLAY_CHANGE_SEQ.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(800));
        if DISPLAY_CHANGE_SEQ.load(Ordering::SeqCst) == seq {
            let _ = app.emit("displays-changed", ());
        }
    });
}

#[tauri::command]
fn get_state(app: AppHandle) -> Result<state::AppState, String> {
    let st = state::read_state(&displayplacer_path(&app))?;
    LAST_DISPLAY_COUNT.store(st.displays.len(), Ordering::SeqCst);
    Ok(st)
}

// 打包后优先用 App 包内自带的 displayplacer，开发环境回落到 PATH
pub fn displayplacer_path(app: &AppHandle) -> String {
    if let Ok(rd) = app.path().resource_dir() {
        for rel in ["binaries/displayplacer", "displayplacer"] {
            let p = rd.join(rel);
            if p.exists() {
                return p.to_string_lossy().into_owned();
            }
        }
    }
    "displayplacer".to_string()
}

fn normalize_res(res: &str) -> String {
    res.replace('×', "x")
}

fn build_spec(d: &ApplyDisplay, ids: &str, origin: &str, include_timing: bool) -> String {
    let mut spec = format!(
        "id:{} res:{} scaling:{} origin:{} degree:{} enabled:{}",
        ids,
        normalize_res(&d.res),
        if d.scaling { "on" } else { "off" },
        origin,
        d.degree,
        if d.enabled { "true" } else { "false" }
    );
    if include_timing {
        if let Some(hz) = d.hz {
            spec.push_str(&format!(" hz:{}", hz));
        }
        if let Some(cd) = d.color_depth {
            spec.push_str(&format!(" color_depth:{}", cd));
        }
    }
    spec
}

fn spec_width(res: &str) -> i32 {
    res.split('x')
        .next()
        .and_then(|w| w.parse().ok())
        .unwrap_or(1920)
}

// v2 角色模型 → displayplacer 多组 spec：
// 镜像组合并为 id:目标+从1+从2（分辨率用目标屏的）；主显示器放 origin:(0,0)
// （displayplacer 借此设置系统主屏），其余组按宽度从左到右排开避免重叠。
fn build_specs(config: &ApplyConfig) -> Result<Vec<String>, String> {
    if config.displays.is_empty() {
        return Err("显示器列表为空".to_string());
    }
    let main_count = config.displays.iter().filter(|d| d.role == "main").count();
    if main_count != 1 {
        return Err(format!("必须且只能有一个主显示器（当前 {main_count} 个）"));
    }
    for d in &config.displays {
        if d.role != "mirror" {
            continue;
        }
        let Some(t) = &d.mirror_target else {
            return Err(format!("镜像屏 {} 缺少镜像目标", d.ctx_id));
        };
        let Some(target) = config.displays.iter().find(|x| &x.ctx_id == t) else {
            return Err("镜像目标不在显示器列表中".to_string());
        };
        if target.role == "mirror" {
            return Err("镜像目标不能自己也是镜像屏".to_string());
        }
    }

    let mut order: Vec<&ApplyDisplay> = Vec::new();
    order.extend(config.displays.iter().filter(|d| d.role == "main"));
    order.extend(config.displays.iter().filter(|d| d.role == "extend"));

    let mut specs = Vec::new();
    let mut x = 0i32;
    for d in order {
        let followers: Vec<&ApplyDisplay> = config
            .displays
            .iter()
            .filter(|f| f.role == "mirror" && f.mirror_target.as_deref() == Some(d.ctx_id.as_str()))
            .collect();
        let mut ids = d.ctx_id.clone();
        for f in &followers {
            ids.push('+');
            ids.push_str(&f.ctx_id);
        }
        let origin = format!("({},{})", x, 0);
        specs.push(build_spec(d, &ids, &origin, followers.is_empty()));
        x += spec_width(&normalize_res(&d.res));
    }
    Ok(specs)
}

fn run_apply(program: &str, specs: &[String]) -> Result<(), String> {
    let out = Command::new(program)
        .args(specs)
        .output()
        .map_err(|e| format!("无法运行 displayplacer：{e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}

#[tauri::command]
fn apply_config(app: AppHandle, config: ApplyConfig) -> Result<(), String> {
    let specs = build_specs(&config)?;
    run_apply(&displayplacer_path(&app), &specs)
}

#[tauri::command]
fn save_preset(app: AppHandle, name: String, config: ApplyConfig) -> Result<(), String> {
    presets::save_preset(&app, &name, &config)
}

#[tauri::command]
fn list_presets(app: AppHandle) -> Result<Vec<presets::PresetEntry>, String> {
    presets::list_presets(&app)
}

#[tauri::command]
fn delete_preset(app: AppHandle, name: String) -> Result<(), String> {
    presets::delete_preset(&app, &name)
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Result<settings::AppSettings, String> {
    settings::get(&app)
}

#[tauri::command]
fn toggle_favorite(
    app: AppHandle,
    display_id: String,
    res: String,
) -> Result<settings::AppSettings, String> {
    settings::toggle_favorite(&app, &display_id, &res)
}

#[tauri::command]
fn set_pin_favorites(app: AppHandle, pin: bool) -> Result<settings::AppSettings, String> {
    settings::set_pin_favorites(&app, pin)
}

#[tauri::command]
fn hide_panel(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

// 设置面板「关于」里的退出：没有 Dock 图标后，这是托盘右键以外的第二个退出入口
#[tauri::command]
fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

// 自启动状态直接读系统里的 LaunchAgent 注册结果，不在 settings.json 里另存一份，
// 免得两处状态对不上（用户手动删了 plist 也要能如实显示「未开启」）
#[tauri::command]
fn get_autostart(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())?;
    } else {
        manager.disable().map_err(|e| e.to_string())?;
    }
    manager.is_enabled().map_err(|e| e.to_string())
}

fn show_panel(app: &AppHandle, view: &str) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let _ = window.emit("tray-view", view);
    let _ = window.show();
    let _ = window.set_focus();
}

fn hide_panel_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let _ = window.hide();
}

fn toggle_panel_at(app: &AppHandle, position: PhysicalPosition<f64>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        hide_panel_window(app);
        return;
    }
    // 托盘点击坐标是物理像素，Retina 下需按 scale 换算逻辑点再定位
    let scale = window.scale_factor().unwrap_or(1.0);
    let width = if LAST_DISPLAY_COUNT.load(Ordering::SeqCst) <= 1 {
        SINGLE_PANEL_WIDTH
    } else {
        PANEL_WIDTH
    };
    let x = (position.x - width * scale / 2.0).max(0.0);
    let y = (position.y + 14.0 * scale).max(0.0);
    let _ = window.set_position(PhysicalPosition::new(x, y));
    let _ = window.set_size(LogicalSize::new(width, PANEL_HEIGHT));
    show_panel(app, "main");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 自启动默认关闭，仅在用户在设置里勾选时写 ~/Library/LaunchAgents 下的 plist
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                // 纯托盘工具：不进 Dock、不占 ⌘Tab；面板弹出时 set_focus 会
                // activateIgnoringOtherApps，仍能正常置前并获得键盘焦点
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
                if let Some(window) = app.get_webview_window("main") {
                    let _ = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None);
                }
                // user_info 持有 AppHandle 的泄漏指针，供 C 回调取用（进程生命周期内有效）
                let ptr = Box::into_raw(Box::new(app.handle().clone())) as *mut c_void;
                unsafe {
                    core_graphics::display::CGDisplayRegisterReconfigurationCallback(
                        on_display_reconfigure,
                        ptr,
                    );
                }
            }

            let show = MenuItem::with_id(app, "show", "显示面板", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "设置…", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &settings, &quit])?;

            // 菜单栏用单色模板 glyph（tools/gen_icons.swift 生成）：template 模式会按菜单栏
            // 深浅自动着色，彩色 App 图标在深色菜单栏下会整块糊进去
            TrayIconBuilder::with_id("main-tray")
                .icon(Image::from_bytes(include_bytes!(
                    "../icons/tray-template.png"
                ))?)
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_panel(app, "main"),
                    "settings" => show_panel(app, "settings"),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        position,
                        ..
                    } = event
                    {
                        toggle_panel_at(tray.app_handle(), position);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            apply_config,
            save_preset,
            list_presets,
            delete_preset,
            get_settings,
            toggle_favorite,
            set_pin_favorites,
            hide_panel,
            quit_app,
            get_autostart,
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
