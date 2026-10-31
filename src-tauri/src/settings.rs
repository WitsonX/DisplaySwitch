use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::lang;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub favorites: HashMap<String, Vec<String>>,
    pub pin_favorites: bool,
    /// 语言偏好："auto" / "zh" / "en"；None（老设置没写过）等同于 "auto"
    pub lang: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Stored {
    favorites: HashMap<String, Vec<String>>,
    // 缺省视为 true：老文件没写过这个字段时保持「收藏置顶」的默认行为
    pin_favorites: Option<bool>,
    lang: Option<String>,
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|e| {
            lang::tr_fill(
                "无法定位配置目录：{e}",
                "Cannot find the config folder: {e}",
                &[("e", &e)],
            )
        })
}

fn load(app: &AppHandle) -> Stored {
    let Ok(path) = settings_path(app) else {
        return Stored::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return Stored::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn store(app: &AppHandle, stored: &Stored) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(stored).map_err(|e| {
        lang::tr_fill(
            "序列化失败：{e}",
            "Could not encode the settings: {e}",
            &[("e", &e)],
        )
    })?;
    fs::write(&path, json).map_err(|e| {
        lang::tr_fill(
            "写入设置失败：{e}",
            "Could not write the settings: {e}",
            &[("e", &e)],
        )
    })
}

fn public(stored: Stored) -> AppSettings {
    AppSettings {
        favorites: stored.favorites,
        pin_favorites: stored.pin_favorites.unwrap_or(true),
        lang: stored.lang,
    }
}

pub fn get(app: &AppHandle) -> Result<AppSettings, String> {
    Ok(public(load(app)))
}

pub fn toggle_favorite(
    app: &AppHandle,
    display_id: &str,
    res: &str,
) -> Result<AppSettings, String> {
    let mut stored = load(app);
    let list = stored.favorites.entry(display_id.to_string()).or_default();
    match list.iter().position(|x| x == res) {
        Some(i) => {
            list.remove(i);
        }
        None => list.push(res.to_string()),
    };
    if list.is_empty() {
        stored.favorites.remove(display_id);
    }
    store(app, &stored)?;
    Ok(public(stored))
}

pub fn set_pin_favorites(app: &AppHandle, pin: bool) -> Result<AppSettings, String> {
    let mut stored = load(app);
    stored.pin_favorites = Some(pin);
    store(app, &stored)?;
    Ok(public(stored))
}

pub fn set_lang(app: &AppHandle, lang: String) -> Result<AppSettings, String> {
    let mut stored = load(app);
    stored.lang = Some(lang);
    store(app, &stored)?;
    Ok(public(stored))
}

/// 生效语言：显式选过 zh/en 就用选的，auto（或没选过）跟随系统；托盘菜单在启动时按它构建
pub fn effective_lang(app: &AppHandle) -> String {
    lang::resolve(load(app).lang.as_deref())
}

#[cfg(test)]
mod tests {
    use super::{public, Stored};

    #[test]
    fn missing_pin_field_reads_as_true() {
        let stored: Stored = serde_json::from_str(r#"{"favorites":{"s1":["1x1"]}}"#).unwrap();
        assert_eq!(stored.pin_favorites, None);
        assert!(public(stored).pin_favorites);
    }

    #[test]
    fn missing_lang_field_means_follow_system() {
        let stored: Stored = serde_json::from_str(r#"{"favorites":{}}"#).unwrap();
        assert_eq!(public(stored).lang, None);
    }

    #[test]
    fn chosen_lang_is_kept() {
        let stored: Stored = serde_json::from_str(r#"{"favorites":{},"lang":"en"}"#).unwrap();
        assert_eq!(public(stored).lang.as_deref(), Some("en"));
    }
}
