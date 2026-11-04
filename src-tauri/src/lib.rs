mod lang;
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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tauri::Emitter;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, LogicalSize, Manager, PhysicalPosition, Position, Size, WindowEvent,
};
use tauri_plugin_global_shortcut::{
    Builder as GlobalShortcutBuilder, GlobalShortcutExt, Shortcut, ShortcutState,
};

const PANEL_WIDTH: f64 = 600.0;
const SINGLE_PANEL_WIDTH: f64 = 380.0;
const PANEL_HEIGHT: f64 = 520.0;

// 单屏时面板用窄窗口；计数由 get_state 刷新，托盘点击时据此选宽度
static LAST_DISPLAY_COUNT: AtomicUsize = AtomicUsize::new(2);

// 快速切换：外部程序（如 MarkLock 便签）带 -quick-switch / -qs 启动时，面板直接进
// 「预设场景优先 + 应用后倒计时退出」。冷启动路径由 argv 置位、前端挂载后
// 一次取走；应用已在托盘驻留时，第二次带参启动由单实例插件转发，只普通弹面板。
static QUICK_SWITCH: AtomicBool = AtomicBool::new(false);
const QUICK_SWITCH_ARGS: [&str; 2] = ["-quick-switch", "-qs"];

fn is_quick_switch_arg(arg: &str) -> bool {
    QUICK_SWITCH_ARGS.contains(&arg)
}

// 快速切换期间（点下场景到倒计时结束）禁止「失焦即隐藏」：displayplacer 重排
// 显示器会让面板瞬间失焦，不锁住的话用户看不到倒计时，也没法点取消
static PANEL_LOCK: AtomicBool = AtomicBool::new(false);

// 托盘「再点一下收起」：点托盘时面板会先失焦触发即时隐藏，等托盘抬起事件到达时窗口
// 已经隐藏，于是又被弹回来，导致第二下收不起。改成失焦后延迟隐藏，这段窗口期内若收到
// 托盘点击就判定为「收起」，取消待隐藏并立即隐藏
static BLUR_HIDE_PENDING: AtomicBool = AtomicBool::new(false);
const BLUR_HIDE_DELAY_MS: u64 = 300;

#[tauri::command]
fn set_panel_lock(locked: bool) -> Result<(), String> {
    PANEL_LOCK.store(locked, Ordering::SeqCst);
    Ok(())
}

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
        return Err(lang::tr("显示器列表为空", "The display list is empty").to_string());
    }
    let main_count = config.displays.iter().filter(|d| d.role == "main").count();
    if main_count != 1 {
        return Err(lang::tr_fill(
            "必须且只能有一个主显示器（当前 {n} 个）",
            "Exactly one main display is required (found {n})",
            &[("n", &main_count)],
        ));
    }
    for d in &config.displays {
        if d.role != "mirror" {
            continue;
        }
        let Some(t) = &d.mirror_target else {
            return Err(lang::tr_fill(
                "镜像屏 {id} 缺少镜像目标",
                "Mirrored display {id} has no mirror target",
                &[("id", &d.ctx_id)],
            ));
        };
        let Some(target) = config.displays.iter().find(|x| &x.ctx_id == t) else {
            return Err(lang::tr(
                "镜像目标不在显示器列表中",
                "The mirror target is not in the display list",
            )
            .to_string());
        };
        if target.role == "mirror" {
            return Err(lang::tr(
                "镜像目标不能自己也是镜像屏",
                "A mirror target cannot be mirrored itself",
            )
            .to_string());
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
        .map_err(|e| {
            lang::tr_fill(
                "无法运行 displayplacer：{e}",
                "Could not run displayplacer: {e}",
                &[("e", &e)],
            )
        })?;
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
fn set_preset_order(
    app: AppHandle,
    order: Vec<String>,
) -> Result<settings::AppSettings, String> {
    settings::set_preset_order(&app, order)
}

#[tauri::command]
fn set_preset_first(app: AppHandle, on: bool) -> Result<settings::AppSettings, String> {
    settings::set_preset_first(&app, on)
}

// 语言存下来后，托盘菜单是原生控件、不会跟着网页重渲染，必须就地重建
#[tauri::command]
fn set_lang(app: AppHandle, lang: String) -> Result<settings::AppSettings, String> {
    let saved = settings::set_lang(&app, lang)?;
    // 存的是偏好，生效值还要把 auto 展开成系统语言，缓存与菜单都按生效值构建
    crate::lang::set_active(&settings::effective_lang(&app));
    sync_tray_menu(&app);
    Ok(saved)
}

#[tauri::command]
fn hide_panel(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

// 设置显示/隐藏主面板的全局快捷键（None / 空串表示清空）。先验格式再动已注册的，
// 免得非法输入把当前可用那个顺手清掉；新键注册失败（多半被别的 App 占用）就回滚旧的
#[tauri::command]
fn set_hotkey(app: AppHandle, shortcut: Option<String>) -> Result<settings::AppSettings, String> {
    let new = shortcut.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let parsed: Option<Shortcut> = match &new {
        Some(s) => Some(s.parse::<Shortcut>().map_err(|_| {
            lang::tr("快捷键格式无效", "Invalid shortcut format").to_string()
        })?),
        None => None,
    };

    let old = settings::load_hotkey(&app);
    let _ = app.global_shortcut().unregister_all();

    if let Some(sc) = parsed {
        if let Err(e) = app.global_shortcut().register(sc) {
            if let Some(o) = old.and_then(|o| o.parse::<Shortcut>().ok()) {
                let _ = app.global_shortcut().register(o);
            }
            return Err(lang::tr_fill(
                "快捷键注册失败，可能已被其他应用占用：{e}",
                "Could not register the shortcut (it may be taken by another app): {e}",
                &[("e", &e)],
            ));
        }
    }
    settings::set_hotkey(&app, new)
}

// 退出：托盘右键菜单、设置面板底部按钮，以及前端拦下来的 ⌘Q 都走这里。
// ⌘Q 只在面板可见时被收到（失焦即收起），不会误退当时那个前台 App
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

// 「检查更新」查到新版后跳发布页。地址是前端拼出来的，但 `open` 把它交给系统浏览器，
// 所以后端只放过我们自己仓库的发布页。
const RELEASE_PAGES: [&str; 2] = [
    "https://github.com/WitsonX/DisplaySwitch/",
    "https://gitee.com/Witson/DisplaySwitch/",
];

fn is_release_page(url: &str) -> bool {
    RELEASE_PAGES.iter().any(|p| url.starts_with(p))
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if !is_release_page(&url) {
        return Err(lang::tr(
            "只允许打开发布页链接",
            "Only release pages can be opened",
        )
        .to_string());
    }
    Command::new("open").arg(&url).spawn().map_err(|e| {
        lang::tr_fill(
            "无法打开浏览器：{e}",
            "Could not open the browser: {e}",
            &[("e", &e)],
        )
    })?;
    Ok(())
}

// 托盘右键菜单：文案按当前界面语言取，语言变了要重新构建一份
fn tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let show = MenuItem::with_id(
        app,
        "show",
        lang::tr("显示面板", "Show Panel"),
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(
        app,
        "settings",
        lang::tr("设置…", "Settings…"),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", lang::tr("退出", "Quit"), true, None::<&str>)?;
    Menu::with_items(app, &[&show, &settings, &quit])
}

fn sync_tray_menu(app: &AppHandle) {
    let Some(tray) = app.tray_by_id("main-tray") else {
        return;
    };
    if let Ok(menu) = tray_menu(app) {
        let _ = tray.set_menu(Some(menu));
    }
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

// 失焦后延迟隐藏：挂起一个待隐藏标志并开定时器，到点时若标志没被托盘点击抢掉才真隐。
// 抢走（swap 置 false）则本次不隐，交给托盘逻辑处理
fn schedule_blur_hide(app: &AppHandle) {
    BLUR_HIDE_PENDING.store(true, Ordering::SeqCst);
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(BLUR_HIDE_DELAY_MS));
        if BLUR_HIDE_PENDING.swap(false, Ordering::SeqCst) {
            hide_panel_window(&app);
        }
    });
}

// 全局快捷键没有托盘坐标做锚点，按单/双屏宽度在主屏居中弹出
fn show_panel_centered(app: &AppHandle, view: &str) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let width = if LAST_DISPLAY_COUNT.load(Ordering::SeqCst) <= 1 {
        SINGLE_PANEL_WIDTH
    } else {
        PANEL_WIDTH
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let _ = window.set_size(LogicalSize::new(width, PANEL_HEIGHT));
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let pos = monitor.position();
        let size = monitor.size();
        let x = (pos.x as f64 + (size.width as f64 - width * scale) / 2.0).max(0.0);
        let y = (pos.y as f64 + (size.height as f64 - PANEL_HEIGHT * scale) / 2.0).max(0.0);
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
    show_panel(app, view);
}

// 快捷键触发：优先贴着托盘图标弹出（和手动点托盘同一落点），拿不到托盘位置时回退主屏居中
fn toggle_panel_hotkey(app: &AppHandle) {
    if let Some(anchor) = tray_anchor(app) {
        // toggle_panel_at 内部已处理「可见则收起」
        toggle_panel_at(app, anchor);
        return;
    }
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        hide_panel_window(app);
    } else {
        show_panel_centered(app, "main");
    }
}

// 托盘图标的水平中心 + 顶边，作为面板锚点（物理像素）。macOS 可取到，
// Linux 不支持或拿不到时返回 None，交给居中回退
fn tray_anchor(app: &AppHandle) -> Option<PhysicalPosition<f64>> {
    let rect = app.tray_by_id("main-tray")?.rect().ok()??;
    // macOS 下 tray rect 是物理像素；Logical 分支极少见，按原值近似处理
    let (px, py) = match rect.position {
        Position::Physical(p) => (p.x as f64, p.y as f64),
        Position::Logical(l) => (l.x, l.y),
    };
    let w = match rect.size {
        Size::Physical(s) => s.width as f64,
        Size::Logical(l) => l.width,
    };
    Some(PhysicalPosition::new(px + w / 2.0, py))
}

fn toggle_panel_at(app: &AppHandle, position: PhysicalPosition<f64>) {
    // 失焦延迟隐藏的窗口期内又点托盘 → 判定为「再点一下收起」：抢掉待隐藏标志，立即隐藏
    if BLUR_HIDE_PENDING.swap(false, Ordering::SeqCst) {
        hide_panel_window(app);
        return;
    }
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

// 快速切换入口：宽度跟随单/多屏（顺带刷新托盘用的计数），主屏居中弹出。
// 托盘弹窗有托盘坐标做锚点，这里没有，居中比固定角落好找
fn show_quick_switch(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let count = state::read_state(&displayplacer_path(app))
        .map(|st| st.displays.len())
        .unwrap_or(2);
    LAST_DISPLAY_COUNT.store(count, Ordering::SeqCst);
    let width = if count <= 1 {
        SINGLE_PANEL_WIDTH
    } else {
        PANEL_WIDTH
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let _ = window.set_size(LogicalSize::new(width, PANEL_HEIGHT));
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let pos = monitor.position();
        let size = monitor.size();
        let x = (pos.x as f64 + (size.width as f64 - width * scale) / 2.0).max(0.0);
        let y = (pos.y as f64 + (size.height as f64 - PANEL_HEIGHT * scale) / 2.0).max(0.0);
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
    let _ = window.show();
    let _ = window.set_focus();
}

// 前端挂载时调用：消费启动参数置的位。返回 true 表示这次是快速切换启动
#[tauri::command]
fn quick_switch_launched(app: AppHandle) -> Result<bool, String> {
    let was = QUICK_SWITCH.swap(false, Ordering::SeqCst);
    if was {
        show_quick_switch(&app);
    }
    Ok(was)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 冷启动路径：argv 里带快速切换参数就置位，等前端挂载后弹面板
    QUICK_SWITCH.store(
        std::env::args_os()
            .any(|a| is_quick_switch_arg(&a.to_string_lossy())),
        Ordering::SeqCst,
    );
    tauri::Builder::default()
        // 自启动默认关闭，仅在用户在设置里勾选时写 ~/Library/LaunchAgents 下的 plist
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // 已在托盘驻留时再次带参启动：第二次进程把参数转给首个实例后自行退出。
        // 这种情形只当普通唤出来用——直接显示面板，不进快速切换
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if args.iter().any(|a| is_quick_switch_arg(a)) {
                show_panel(app, "main");
            }
        }))
        // 全局快捷键：用户在设置里自定义“显示/隐藏主面板”，默认空。处理器只负责切换面板，
        // 具体注册哪个键由 set_hotkey 命令与启动恢复逻辑控制（同一时刻最多一个）
        .plugin(
            GlobalShortcutBuilder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        toggle_panel_hotkey(app);
                    }
                })
                .build(),
        )
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

            // 原生侧文案先定语言，再建菜单
            lang::set_active(&settings::effective_lang(app.handle()));
            let menu = tray_menu(app.handle())?;

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

            // 恢复上次保存的全局快捷键（默认为空，没设过就不注册）；解析失败静默跳过
            if let Some(hk) = settings::load_hotkey(app.handle()) {
                if let Ok(sc) = hk.parse::<Shortcut>() {
                    let _ = app.handle().global_shortcut().register(sc);
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::Focused(false) = event {
                    // 只在还可见时排延迟隐藏；自己 hide() 触发的后续失焦事件此时已不可见，不重复挂起
                    if !PANEL_LOCK.load(Ordering::SeqCst) && window.is_visible().unwrap_or(false) {
                        schedule_blur_hide(window.app_handle());
                    }
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
            set_preset_order,
            set_preset_first,
            set_lang,
            hide_panel,
            set_hotkey,
            quit_app,
            quick_switch_launched,
            set_panel_lock,
            get_autostart,
            set_autostart,
            open_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::is_release_page;

    #[test]
    fn only_our_release_pages_are_allowed() {
        assert!(is_release_page("https://gitee.com/Witson/DisplaySwitch/releases/tag/0.3.0"));
        assert!(is_release_page(
            "https://github.com/WitsonX/DisplaySwitch/releases/tag/v0.3.0"
        ));
        assert!(!is_release_page("https://evil.test/WitsonX/DisplaySwitch/releases"));
        assert!(!is_release_page("http://github.com/WitsonX/DisplaySwitch/releases/tag/v1"));
        assert!(!is_release_page("https://github.com/WitsonX/DisplaySwitchX/releases"));
    }

    // 前端录入产出的规范串（修饰符 + keyboard-types Code 名）必须能被插件解析，
    // 否则注册时才报错、用户只会看到一句莫名的失败
    #[test]
    fn frontend_hotkey_strings_parse() {
        use super::Shortcut;
        for s in [
            "Command+Shift+D",
            "Command+1",
            "Alt+Space",
            "Command+Control+Alt+Shift+F5",
            "Command+ArrowUp",
            "Command+Comma",
        ] {
            assert!(s.parse::<Shortcut>().is_ok(), "{s} should parse");
        }
        assert!("Shift+A".parse::<Shortcut>().is_ok());
        assert!("NotAKey+Foo".parse::<Shortcut>().is_err());
    }
}
