#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Ja,
    En,
}

#[allow(dead_code)]
fn parse_lang_override(value: Option<String>) -> Option<Lang> {
    value.and_then(|val| match val.to_lowercase().as_str() {
        "ja" => Some(Lang::Ja),
        "en" => Some(Lang::En),
        _ => None,
    })
}

#[allow(dead_code)]
fn lang_from_locale_str(locale: &str) -> Lang {
    if locale.to_lowercase().starts_with("ja") {
        Lang::Ja
    } else {
        Lang::En
    }
}

#[allow(dead_code)]
pub fn resolve_lang() -> Lang {
    if let Some(lang) = parse_lang_override(std::env::var("BWQA_LANG").ok()) {
        return lang;
    }
    if let Some(locale) = sys_locale::get_locale() {
        return lang_from_locale_str(&locale);
    }
    Lang::En
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lang_override() {
        assert_eq!(parse_lang_override(Some("ja".to_string())), Some(Lang::Ja));
        assert_eq!(parse_lang_override(Some("en".to_string())), Some(Lang::En));
        assert_eq!(parse_lang_override(Some("fr".to_string())), None);
        assert_eq!(parse_lang_override(None), None);

        // 大文字混じりの扱い
        assert_eq!(parse_lang_override(Some("JA".to_string())), Some(Lang::Ja));
        assert_eq!(parse_lang_override(Some("En".to_string())), Some(Lang::En));
        assert_eq!(parse_lang_override(Some("eN".to_string())), Some(Lang::En));
    }

    #[test]
    fn test_lang_from_locale_str() {
        assert_eq!(lang_from_locale_str("ja-JP"), Lang::Ja);
        assert_eq!(lang_from_locale_str("ja_JP"), Lang::Ja);
        assert_eq!(lang_from_locale_str("en-US"), Lang::En);
        assert_eq!(lang_from_locale_str("fr-FR"), Lang::En);

        // ケースインセンシティブ判定のテスト
        assert_eq!(lang_from_locale_str("JA-JP"), Lang::Ja);
        assert_eq!(lang_from_locale_str("Ja_JP"), Lang::Ja);
        assert_eq!(lang_from_locale_str("EN-US"), Lang::En);
    }
}
