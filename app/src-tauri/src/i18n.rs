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

/// 表示言語を「BWQA_LANG環境変数 → OSシステムロケール → フォールバック(En)」の順で判定する。
///
/// `BWQA_LANG` はあくまで開発時(`cargo run` 等、ターミナルから起動する場合)の補助的な
/// オーバーライド手段である。macOSのGUIアプリはFinderからの起動やログイン項目からの
/// 自動起動ではシェルの環境変数を継承しないため、パッケージ済み `.app` の通常利用では
/// 機能しない。エンドユーザー向けの言語切り替え手段としては案内しないこと(design.md 決定3)。
pub fn resolve_lang() -> Lang {
    if let Some(lang) = parse_lang_override(std::env::var("BWQA_LANG").ok()) {
        return lang;
    }
    if let Some(locale) = sys_locale::get_locale() {
        return lang_from_locale_str(&locale);
    }
    Lang::En
}

pub struct Messages {
    pub status_disconnected: &'static str,
    pub status_locked: &'static str,
    pub status_unlocked: &'static str,
    pub hotkey_registered: &'static str,
    /// `{}` に hotkey_registration_failed の展開結果を埋め込むテンプレート
    pub hotkey_unregistered_prefix: &'static str,
    /// `{}` に元のエラー(`err`)を埋め込むテンプレート
    pub hotkey_registration_failed: &'static str,
    pub autostart_label: &'static str,
    pub quit_label: &'static str,
    pub repo_link_label: &'static str,
}

pub const JA: Messages = Messages {
    status_disconnected: "状態: 未接続",
    status_locked: "状態: ロック中",
    status_unlocked: "状態: アンロック済み",
    hotkey_registered: "ホットキー: ⇧⌘Space",
    hotkey_unregistered_prefix: "⚠ ホットキー未登録: {}",
    hotkey_registration_failed: "グローバルホットキー(Shift+Cmd+Space)の登録に失敗しました: {}\n他のアプリと衝突しているか、macOSのシステム設定 > プライバシーとセキュリティ > アクセシビリティ でこのアプリの権限が許可されていない可能性があります。",
    autostart_label: "ログイン時に自動起動",
    quit_label: "終了",
    repo_link_label: "GitHubリポジトリを開く",
};

pub const EN: Messages = Messages {
    status_disconnected: "Status: Disconnected",
    status_locked: "Status: Locked",
    status_unlocked: "Status: Unlocked",
    hotkey_registered: "Hotkey: ⇧⌘Space",
    hotkey_unregistered_prefix: "⚠ Hotkey not registered: {}",
    hotkey_registration_failed: "Failed to register the global hotkey (Shift+Cmd+Space): {}\nIt may conflict with another app, or this app may not have Accessibility permission under System Settings > Privacy & Security > Accessibility.",
    autostart_label: "Launch at Login",
    quit_label: "Quit",
    repo_link_label: "View on GitHub",
};

pub fn messages(lang: Lang) -> &'static Messages {
    match lang {
        Lang::Ja => &JA,
        Lang::En => &EN,
    }
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
