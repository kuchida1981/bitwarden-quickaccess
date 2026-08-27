const MESSAGES = {
  ja: {
    appDisplayName: "Bitwarden クイックアクセス",
    errorScreenTitle: "接続できません",
    masterPasswordPlaceholder: "マスターパスワード",
    unlockButton: "アンロック",
    searchPlaceholder: "検索...",
    emptyMessage: "アイテムが見つかりません",
    shortcutHints: "⌘C ユーザー名 / ⌘⇧C パスワード / ⌥⌘C TOTP / Enter ブラウザで開く / → メニュー",
    unlockFailed: "アンロックに失敗しました。",
    actionCopyUsername: "ユーザー名をコピー",
    actionCopyPassword: "パスワードをコピー",
    actionCopyTotp: "ワンタイムパスワードをコピー",
    actionOpenBrowser: "ブラウザで開く",
    helpTitle: "ショートカット",
    helpTogglePopup: "クイックアクセスを表示/非表示",
    helpMoveFocus: "アイテム間を移動",
    helpOpenMenu: "アクションメニューを開く",
    helpCloseMenu: "アクションメニューを閉じる",
    helpCopyUsername: "ユーザー名をコピー",
    helpCopyPassword: "パスワードをコピー",
    helpCopyTotp: "ワンタイムパスワードをコピー",
    helpOpenBrowser: "ブラウザで開く",
    helpLock: "ロック",
    helpClose: "閉じる",
    helpToggleHelp: "このヘルプを表示/非表示",
  },
  en: {
    appDisplayName: "Bitwarden Quick Access",
    errorScreenTitle: "Connection Error",
    masterPasswordPlaceholder: "Master password",
    unlockButton: "Unlock",
    searchPlaceholder: "Search...",
    emptyMessage: "No items found",
    shortcutHints: "⌘C Username / ⌘⇧C Password / ⌥⌘C TOTP / Enter Open in browser / → Menu",
    unlockFailed: "Failed to unlock.",
    actionCopyUsername: "Copy Username",
    actionCopyPassword: "Copy Password",
    actionCopyTotp: "Copy One-Time Password",
    actionOpenBrowser: "Open in Browser",
    helpTitle: "Shortcuts",
    helpTogglePopup: "Toggle Quick Access",
    helpMoveFocus: "Move between items",
    helpOpenMenu: "Open action menu",
    helpCloseMenu: "Close action menu",
    helpCopyUsername: "Copy username",
    helpCopyPassword: "Copy password",
    helpCopyTotp: "Copy one-time password",
    helpOpenBrowser: "Open in browser",
    helpLock: "Lock",
    helpClose: "Close",
    helpToggleHelp: "Toggle this help",
  },
};

let currentLocale = "en";

function t(key) {
  return (MESSAGES[currentLocale] && MESSAGES[currentLocale][key]) || MESSAGES.ja[key] || key;
}

function applyStaticI18n() {
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    el.textContent = t(el.getAttribute("data-i18n"));
  });
  document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
    el.setAttribute("placeholder", t(el.getAttribute("data-i18n-placeholder")));
  });
}

async function initI18n() {
  try {
    const locale = await window.__TAURI__.core.invoke("get_ui_locale");
    if (MESSAGES[locale]) {
      currentLocale = locale;
    }
  } catch {
    // 取得に失敗した場合は既定(en、Rust側resolve_lang()の最終フォールバックと合わせる)のまま続行する
  }
  applyStaticI18n();
}
