use std::sync::atomic::{AtomicBool, Ordering};

/// 界面语言在原生侧的缓存。托盘菜单和所有错误提示都要读它，
/// 把 lang 一路穿到每个调用点太碎，所以只放一个原子布尔。
static EN: AtomicBool = AtomicBool::new(false);

/// AppleLanguages 的输出里，语言标识被这些字符隔开
const SEPARATORS: [char; 8] = [' ', '\t', '\n', '\r', ',', '(', ')', '"'];

pub fn set_active(lang: &str) {
    EN.store(lang == "en", Ordering::Relaxed);
}

pub fn is_en() -> bool {
    EN.load(Ordering::Relaxed)
}

/// 取一条当前语言的文案
pub fn tr<'a>(zh: &'a str, en: &'a str) -> &'a str {
    if is_en() {
        en
    } else {
        zh
    }
}

/// 双语错误串带占位符：模板里的 {name} / {e} 换成实际内容（与前端 i18n 同一套写法）
pub fn tr_fill(zh: &str, en: &str, args: &[(&str, &dyn std::fmt::Display)]) -> String {
    let mut text = tr(zh, en).to_string();
    for (key, value) in args {
        text = text.replace(&format!("{{{key}}}"), &value.to_string());
    }
    text
}

/// 语言偏好三态：显式选过 zh / en 就用选的，auto（或老设置里没这个键）跟随系统首选语言
pub fn resolve(pref: Option<&str>) -> String {
    match pref {
        Some(lang @ ("zh" | "en")) => lang.to_string(),
        _ => system_lang(),
    }
}

/// macOS 的首选语言在 AppleLanguages 数组第一项；读不到就按 App 的原生语言中文
fn system_lang() -> String {
    let Ok(out) = std::process::Command::new("defaults")
        .args(["read", "-globalDomain", "AppleLanguages"])
        .output()
    else {
        return "zh".to_string();
    };
    first_locale(&String::from_utf8_lossy(&out.stdout)).unwrap_or_else(|| "zh".to_string())
}

fn first_locale(text: &str) -> Option<String> {
    text.split(SEPARATORS)
        .find(|token| !token.is_empty())
        .map(|token| {
            if token.starts_with("zh") {
                "zh".to_string()
            } else {
                "en".to_string()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{first_locale, resolve};

    #[test]
    fn reads_the_first_preferred_language() {
        assert_eq!(first_locale("(\n    \"zh-Hans-CN\",\n    \"en-CN\",\n)"), Some("zh".into()));
        assert_eq!(first_locale("(\n    en-US,\n)"), Some("en".into()));
        assert_eq!(first_locale("(\n)"), None);
    }

    #[test]
    fn explicit_prefs_are_used_as_is() {
        assert_eq!(resolve(Some("zh")), "zh");
        assert_eq!(resolve(Some("en")), "en");
    }

    #[test]
    fn auto_and_unknown_prefs_fall_back_to_the_system() {
        for pref in [None, Some("auto"), Some("fr-FR")] {
            assert!(matches!(resolve(pref).as_str(), "zh" | "en"));
        }
    }
}
