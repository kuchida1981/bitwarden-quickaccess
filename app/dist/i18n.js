const MESSAGES = {
  ja: {
    masterPasswordPlaceholder: "マスターパスワード",
    unlockButton: "アンロック",
    searchPlaceholder: "検索...",
    emptyMessage: "アイテムが見つかりません",
    shortcutHints: "⌘C ユーザー名 / ⌘⇧C パスワード / ⌥⌘C TOTP / Enter ブラウザで開く",
    unlockFailed: "アンロックに失敗しました。",
    openedInBrowser: "ブラウザで開きました",
    copiedUsername: "ユーザー名をコピーしました",
    copiedPassword: "パスワードをコピーしました",
    copiedTotp: "TOTPコードをコピーしました",
    actionFailed: "操作に失敗しました。",
  },
  en: {
    masterPasswordPlaceholder: "Master password",
    unlockButton: "Unlock",
    searchPlaceholder: "Search...",
    emptyMessage: "No items found",
    shortcutHints: "⌘C Username / ⌘⇧C Password / ⌥⌘C TOTP / Enter Open in browser",
    unlockFailed: "Failed to unlock.",
    openedInBrowser: "Opened in browser",
    copiedUsername: "Username copied",
    copiedPassword: "Password copied",
    copiedTotp: "TOTP code copied",
    actionFailed: "Action failed.",
  },
};

let currentLocale = "ja";

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
    // 取得に失敗した場合は既定(ja)のまま続行する
  }
  applyStaticI18n();
}
