//! Launcher text follows the Android system language. The community shipped the
//! launcher UI with Simplified Chinese baked into the Rust literals; wrapping
//! them in [`tr`] keeps both the original English and the Chinese, so a Chinese
//! system shows Chinese and every other system shows the original English.
//!
//! The browser, catalog and other game text use `horizon_client::lang` and its
//! `lang/<code>.json` tables. This module is only for the launcher's own
//! hardcoded strings, which never go through that table.
use std::sync::atomic::{AtomicU8, Ordering};

/// 0 = no system language reported yet (English), 1 = Chinese, 2 = other.
static LANGUAGE: AtomicU8 = AtomicU8::new(0);

/// Called from JNI (`StarlingActivity.onCreate`) with `Locale.getDefault().getLanguage()`.
pub fn set_system_language(language: &str) {
    let chinese = language.starts_with("zh");
    LANGUAGE.store(if chinese { 1 } else { 2 }, Ordering::Relaxed);
}

pub fn is_chinese() -> bool {
    LANGUAGE.load(Ordering::Relaxed) == 1
}

/// The game-side `lang` code for the device language, so the setup screens (which
/// use `horizon_client::lang`, not [`tr`]) follow the system too.
pub fn system_lang_code() -> &'static str {
    if is_chinese() { "zh_cn" } else { "en_us" }
}

/// The Chinese text on a Chinese system, the English original everywhere else.
pub fn tr(english: &'static str, chinese: &'static str) -> &'static str {
    if is_chinese() { chinese } else { english }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_follows_the_system_locale() {
        set_system_language("zh-CN");
        assert!(is_chinese());
        assert_eq!(tr("Settings", "设置"), "设置");
        set_system_language("en");
        assert!(!is_chinese());
        assert_eq!(tr("Settings", "设置"), "Settings");
        set_system_language("");
    }
}
