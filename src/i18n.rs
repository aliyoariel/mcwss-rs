//! Tiny i18n layer: locale files are embedded into the binary.

use std::collections::HashMap;

pub const DEFAULT_LANGUAGE: &str = "en_US";
pub const SUPPORTED_LANGUAGES: [&str; 4] = ["en_US", "ja_JP", "zh_TW", "zh_CN"];

pub struct I18n {
    map: HashMap<String, String>,
}

impl I18n {
    /// Unknown languages fall back to [`DEFAULT_LANGUAGE`].
    pub fn new(lang: &str) -> Self {
        let raw = match lang {
            "ja_JP" => include_str!("../locales/ja_JP.json"),
            "zh_TW" => include_str!("../locales/zh_TW.json"),
            "zh_CN" => include_str!("../locales/zh_CN.json"),
            _ => include_str!("../locales/en_US.json"),
        };
        let map = serde_json::from_str(raw).unwrap_or_default();
        Self { map }
    }

    /// Translate a key; unknown keys are returned unchanged.
    pub fn locale(&self, key: &str) -> String {
        self.map
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    /// Translate `template`, then replace `{0}`, `{1}`, ... with the
    /// (translated) arguments.
    pub fn format(&self, template: &str, args: &[String]) -> String {
        let mut content = self.locale(template);
        for (index, value) in args.iter().enumerate() {
            content = content.replace(&format!("{{{index}}}"), &self.locale(value));
        }
        content
    }
}
