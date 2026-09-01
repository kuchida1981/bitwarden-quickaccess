const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const unlockScreen = document.getElementById("unlock-screen");
const errorScreen = document.getElementById("error-screen");
const errorMessage = document.getElementById("error-message");
const searchScreen = document.getElementById("search-screen");
const unlockForm = document.getElementById("unlock-form");
const passwordInput = document.getElementById("master-password");
const unlockError = document.getElementById("unlock-error");
const unlockButton = unlockForm.querySelector("button");
const searchBox = document.getElementById("search-box");
const resultsList = document.getElementById("results");
const emptyMessage = document.getElementById("empty-message");
const statusFooter = document.getElementById("status-footer");
const currentUserAvatar = document.getElementById("current-user-avatar");
const footerHints = document.getElementById("footer-hints");
const helpOverlay = document.getElementById("help-overlay");

const SEARCH_DEBOUNCE_MS = 150;
let SHORTCUT_HINTS = "";
let currentPlatform = "macos";

let currentItems = [];
let focusedIndex = -1;
let debounceTimer = null;
let searchRequestId = 0;
let lastKnownScreen = "unlock";
let hiddenAt = null;

const SEARCH_STATE_RETENTION_MS = 30000;

function shouldRetainSearchState() {
  return hiddenAt !== null && Date.now() - hiddenAt < SEARCH_STATE_RETENTION_MS;
}

let actionMenuOpen = false;
let actionMenuActions = [];
let actionMenuFocusIndex = -1;
let helpOpen = false;

function showScreen(name) {
  unlockScreen.classList.toggle("active", name === "unlock");
  errorScreen.classList.toggle("active", name === "error");
  searchScreen.classList.toggle("active", name === "search");
}

// フォーカス行を点滅させるヘルパー(design.md 決定1)。
// アニメーション完了後に自動でクラスを外す(tasks 1.2)。
function flashRow(element) {
  return new Promise((resolve) => {
    if (!element) {
      resolve();
      return;
    }
    const onEnd = () => {
      element.removeEventListener("animationend", onEnd);
      element.classList.remove("flash");
      resolve();
    };
    element.addEventListener("animationend", onEnd, { once: true });
    element.classList.add("flash");
  });
}

// バックエンド接続エラーのメッセージ取得。取得失敗時は空文字を返す。
async function fetchBackendError() {
  try {
    return (await invoke("get_backend_error")) || "";
  } catch {
    return "";
  }
}

async function refreshCurrentUser() {
  let email = null;
  try {
    email = await invoke("get_current_user");
  } catch {
    email = null;
  }
  if (email) {
    currentUserAvatar.textContent = email.charAt(0).toUpperCase();
    currentUserAvatar.title = email;
    currentUserAvatar.style.display = "flex";
  } else {
    currentUserAvatar.textContent = "";
    currentUserAvatar.title = "";
    currentUserAvatar.style.display = "none";
  }
}

async function syncScreenWithBackend() {
  let lockState = "disconnected";
  try {
    lockState = await invoke("get_lock_state");
  } catch {
    // 取得に失敗した場合はアンロックフォーム側にフォールバックする
  }

  // disconnectedはバックエンド起動直後(preflight/bw serve接続確認中)にも
  // 一時的に取り得る状態で、その間はまだlast_errorが記録されていない。
  // 実際にエラーメッセージが記録されている場合のみ専用のエラー画面を表示し、
  // それ以外(起動中の一時的なdisconnected)は従来通りアンロック画面に
  // フォールバックする(issue #79の対象は「エラーが起きて放置される」ケース)。
  let backendError = "";
  if (lockState === "disconnected") {
    backendError = await fetchBackendError();
  }
  const actualScreen =
    lockState === "unlocked" ? "search" : lockState === "locked" || !backendError ? "unlock" : "error";

  if (actualScreen !== "search") {
    actionMenuOpen = false;
    actionMenuActions = [];
    actionMenuFocusIndex = -1;
    helpOpen = false;
    helpOverlay.classList.remove("visible");
  }

  if (actualScreen === lastKnownScreen) {
    if (actualScreen === "search") {
      if (!shouldRetainSearchState()) {
        searchBox.value = "";
        await runSearch("");
      }
      await refreshCurrentUser();
    } else if (actualScreen === "error") {
      errorMessage.textContent = backendError;
    }
  } else {
    showScreen(actualScreen);
    if (actualScreen === "search") {
      searchBox.value = "";
      searchBox.focus();
      await runSearch("");
      await refreshCurrentUser();
    } else if (actualScreen === "unlock") {
      passwordInput.value = "";
      unlockError.textContent = "";
      passwordInput.focus();
    } else if (actualScreen === "error") {
      errorMessage.textContent = backendError;
    }
  }

  lastKnownScreen = actualScreen;
}

async function handleShown() {
  actionMenuOpen = false;
  actionMenuActions = [];
  actionMenuFocusIndex = -1;
  helpOpen = false;
  helpOverlay.classList.remove("visible");
  showScreen(lastKnownScreen);
  if (lastKnownScreen === "search") {
    searchBox.focus();
    if (shouldRetainSearchState()) {
      searchBox.select();
    }
  } else if (lastKnownScreen === "unlock") {
    passwordInput.value = "";
    unlockError.textContent = "";
    passwordInput.focus();
  }

  await syncScreenWithBackend();
}

unlockForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const password = passwordInput.value;
  if (!password) {
    return;
  }

  unlockButton.disabled = true;
  passwordInput.disabled = true;
  unlockError.textContent = "";

  try {
    await invoke("unlock", { password });
    await syncScreenWithBackend();
  } catch (err) {
    unlockError.textContent = typeof err === "string" ? err : t("unlockFailed");
    passwordInput.value = "";
  } finally {
    unlockButton.disabled = false;
    passwordInput.disabled = false;
  }

  if (unlockScreen.classList.contains("active")) {
    passwordInput.focus();
  }
});

searchBox.addEventListener("input", () => {
  clearTimeout(debounceTimer);
  const query = searchBox.value;
  debounceTimer = setTimeout(() => runSearch(query), SEARCH_DEBOUNCE_MS);
});

// macOSでは⌘(metaKey)、それ以外(Linux等)ではCtrl(ctrlKey)をプライマリの
// モディファイアキーとして扱う(design.md 決定2)。ヘルプ/手動ロック/コピー系
// ショートカットの判定箇所すべてから共通で参照する。
function isPrimaryMod(event) {
  return currentPlatform === "macos" ? event.metaKey : event.ctrlKey;
}

// macOS表記(⌘/⇧/⌥)を基準の正規形として保持し、macOS以外ではCtrl+/Shift+/Alt+
// 表記に変換する(design.md 決定3)。⌘→⇧→⌥の置換順によらず結果は一意に定まる。
function formatShortcutForPlatform(macLabel) {
  if (currentPlatform === "macos") {
    return macLabel;
  }
  return macLabel.replace(/⌘/g, "Ctrl+").replace(/⌥/g, "Alt+").replace(/⇧/g, "Shift+");
}

// 実行プラットフォームを取得し、以降のモディファイアキー判定・ショートカット表記
// 生成に用いる(design.md 決定1)。取得に失敗した場合は既定(macos)のまま続行する。
async function initPlatform() {
  try {
    currentPlatform = await invoke("get_platform");
  } catch {
    // 既定(macos)のまま続行する
  }
}

// ヘルプオーバーレイ内の`data-mod-kbd`付き<kbd>バッジをプラットフォームに応じた
// 表記に更新する(design.md 決定3)。グローバルホットキー(⇧⌘Space)の行は本changeの
// スコープ外(Issue #147)のため対象外とし、`data-mod-kbd`属性を付与していない。
function updateHelpOverlayKbd() {
  document.querySelectorAll("[data-mod-kbd]").forEach((kbd) => {
    kbd.textContent = formatShortcutForPlatform(kbd.getAttribute("data-mod-kbd"));
  });
}

// event.key ではなく event.code(物理キー位置)で判定する(handleActionShortcutの
// ⌘Cと同じ理由)。ヘルプの開閉どちらからも参照するため共通関数にしておく。
function isHelpToggleShortcut(event) {
  return isPrimaryMod(event) && event.code === "Slash" && !event.shiftKey && !event.altKey;
}

// フォーカス要素(検索ボックス/パスワード入力欄等)に関わらずEscapeキーで
// ポップアップやオーバーレイを閉じられるよう、documentレベルで集約して処理する(issue #76)。
document.addEventListener("keydown", (event) => {
  if (event.key !== "Escape") {
    return;
  }
  if (helpOpen) {
    event.preventDefault();
    closeHelp();
    return;
  }
  if (actionMenuOpen) {
    event.preventDefault();
    closeActionMenu();
    return;
  }
  event.preventDefault();
  invoke("hide_popup").catch(() => {});
});

searchBox.addEventListener("keydown", (event) => {
  if (helpOpen) {
    handleHelpKeydown(event);
    return;
  }
  // アクションメニュー表示中でも⌘/は優先する(design.md 決定4: メニューを
  // 閉じてからヘルプを表示する)。actionMenuOpenの分岐より先に判定する必要がある。
  if (isHelpToggleShortcut(event)) {
    event.preventDefault();
    openHelp();
    return;
  }
  if (isPrimaryMod(event) && event.code === "KeyL") {
    event.preventDefault();
    performLock();
    return;
  }
  if (actionMenuOpen) {
    handleActionMenuKeydown(event);
    return;
  }

  if (event.key === "ArrowDown") {
    event.preventDefault();
    moveFocus(1);
    return;
  }
  if (event.key === "ArrowUp") {
    event.preventDefault();
    moveFocus(-1);
    return;
  }
  if (event.key === "ArrowRight") {
    event.preventDefault();
    openActionMenu();
    return;
  }
  handleActionShortcut(event);
});

// 直近の「本物の」マウスカーソル位置(#128)。scrollIntoView() 等のレイアウト変更で
// カーソル直下の要素が変わっても、この座標はブラウザが合成する mouseenter/mouseover では
// 更新されない(mousemove は実際にポインタが物理的に動いた場合にのみ発火するため)。
// 各行の mouseenter ハンドラは、この座標と発火時の座標を比較することで、本物の移動による
// 進入か、レイアウト変更による亡霊イベントかを区別する。
let lastRealMouseX = null;
let lastRealMouseY = null;

resultsList.addEventListener("mousemove", (event) => {
  lastRealMouseX = event.clientX;
  lastRealMouseY = event.clientY;
});

function moveFocus(delta) {
  if (currentItems.length === 0) {
    return;
  }
  const previousIndex = focusedIndex;
  focusedIndex = Math.min(Math.max(focusedIndex + delta, 0), currentItems.length - 1);
  updateFocusRows(previousIndex);
}

// 選択中アイテムの実行可能アクション一覧を組み立てる(design.md 決定4)。
// フィールドを持たない項目は結果配列から除外する(#52)。
function buildActionsForItem(item) {
  const actions = [
    {
      key: "username",
      labelKey: "actionCopyUsername",
      shortcutHint: formatShortcutForPlatform("⌘C"),
      enabled: !!item.username,
    },
    {
      key: "password",
      labelKey: "actionCopyPassword",
      shortcutHint: formatShortcutForPlatform("⌘⇧C"),
      enabled: item.has_password,
    },
    {
      key: "totp",
      labelKey: "actionCopyTotp",
      shortcutHint: formatShortcutForPlatform("⌥⌘C"),
      enabled: item.has_totp,
    },
    { key: "browser", labelKey: "actionOpenBrowser", shortcutHint: "Enter", enabled: item.has_url },
  ];
  return actions.filter((action) => action.enabled);
}

function openActionMenu() {
  if (currentItems.length === 0 || focusedIndex < 0) {
    return;
  }
  const actions = buildActionsForItem(currentItems[focusedIndex]);
  if (actions.length === 0) {
    return;
  }
  actionMenuActions = actions;
  actionMenuOpen = true;
  actionMenuFocusIndex = 0;
  refreshFocusedRowTrailing();
}

function closeActionMenu() {
  actionMenuOpen = false;
  actionMenuActions = [];
  actionMenuFocusIndex = -1;
  refreshFocusedRowTrailing();
}

// アクションメニュー展開中のキー操作(design.md 決定3)。
// ⌘C系のダイレクトショートカットはメニュー表示中も従来通り動作させ、
// それ以外の未処理キー(検索文字入力等)はメニューの前提が崩れないよう無視する。
function handleActionMenuKeydown(event) {
  if (event.key === "ArrowDown") {
    event.preventDefault();
    actionMenuFocusIndex = Math.min(actionMenuFocusIndex + 1, actionMenuActions.length - 1);
    refreshFocusedRowTrailing();
    return;
  }
  if (event.key === "ArrowUp") {
    event.preventDefault();
    actionMenuFocusIndex = Math.max(actionMenuFocusIndex - 1, 0);
    refreshFocusedRowTrailing();
    return;
  }
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    closeActionMenu();
    return;
  }
  if (event.key === "Enter") {
    event.preventDefault();
    const action = actionMenuActions[actionMenuFocusIndex];
    if (action) {
      executeItemAction(currentItems[focusedIndex], action.key);
    }
    return;
  }

  handleActionShortcut(event);
  if (!event.defaultPrevented) {
    event.preventDefault();
  }
}

function openHelp() {
  if (actionMenuOpen) {
    closeActionMenu();
  }
  helpOpen = true;
  helpOverlay.classList.add("visible");
}

function closeHelp() {
  helpOpen = false;
  helpOverlay.classList.remove("visible");
}

// ヘルプ表示中のキー操作(design.md 決定3)。
// Escapeまたは⌘/でヘルプを閉じ、それ以外のキーはすべて無視して
// 検索文字入力などを抑止する。
function handleHelpKeydown(event) {
  if (isHelpToggleShortcut(event)) {
    event.preventDefault();
    closeHelp();
    return;
  }

  event.preventDefault();
}

// 検索ボックスにテキスト選択がある間は、通常のOS標準コピー動作を妨げない
// (design.md 決定2)。選択がない場合のみ `⌘C` 系をアクションとして扱う。
function hasTextSelectionInSearchBox() {
  return searchBox.selectionStart !== searchBox.selectionEnd;
}

// アイテムに対するアクション実行を一箇所に集約する。ダイレクトショートカット
// (`handleActionShortcut`)とアクションメニュー(`handleActionMenuKeydown`/クリック)
// の両方から呼ばれる(design.md 決定4、実装の重複を避ける)。
function executeItemAction(item, key) {
  if (!item) {
    return;
  }
  switch (key) {
    case "username":
      runAction(() => invoke("copy_field", { itemId: item.id, field: "username" }));
      break;
    case "password":
      runAction(() => invoke("copy_field", { itemId: item.id, field: "password" }));
      break;
    case "totp":
      runAction(() => invoke("copy_field", { itemId: item.id, field: "totp" }));
      break;
    case "browser":
      runAction(() => invoke("open_in_browser", { itemId: item.id }));
      break;
    default:
      break;
  }
}

function handleActionShortcut(event) {
  if (event.isComposing) {
    return;
  }
  if (currentItems.length === 0 || focusedIndex < 0) {
    return;
  }
  const item = currentItems[focusedIndex];

  if (event.key === "Enter") {
    event.preventDefault();
    executeItemAction(item, "browser");
    return;
  }

  // event.key ではなく event.code(物理キー位置)で判定する。
  // macOSではOptionキーを押しながら文字キーを押すと、event.key の値が
  // 合成された特殊文字(例: Option+C -> "ç")になり、"c" と一致しなくなるため。
  if (!isPrimaryMod(event) || event.code !== "KeyC") {
    return;
  }

  if (!event.shiftKey && !event.altKey) {
    if (hasTextSelectionInSearchBox()) {
      return;
    }
    event.preventDefault();
    executeItemAction(item, "username");
  } else if (event.shiftKey && !event.altKey) {
    event.preventDefault();
    executeItemAction(item, "password");
  } else if (event.altKey && !event.shiftKey) {
    event.preventDefault();
    executeItemAction(item, "totp");
  }
}

// ⌘Lによる明示的ロック(design.md参照、issue #66)。ロック後はポップアップが
// 表示されていれば即座にアンロック画面に切り替える。
async function performLock() {
  try {
    await invoke("lock");
  } catch {
    return;
  }
  await syncScreenWithBackend();
}

// 1Password Quick Accessに倣い、入力受付の合図としてフォーカス行を点滅させ、
// 成功時のみポップアップを閉じる(design.md 決定1・2)。失敗時は閉じずに検索画面にとどまる。
async function runAction(actionFn) {
  const focusedEl = resultsList.children[focusedIndex];
  const flashPromise = flashRow(focusedEl);
  let ok = true;
  try {
    await actionFn();
  } catch {
    ok = false;
  }

  await flashPromise;

  if (ok) {
    invoke("hide_popup").catch(() => {});
  }
}

async function runSearch(query) {
  const requestId = ++searchRequestId;
  let items = [];
  try {
    items = await invoke("search_items", { query });
  } catch {
    items = [];
  }
  if (requestId !== searchRequestId) {
    return;
  }
  currentItems = items;
  focusedIndex = items.length > 0 ? 0 : -1;
  // 新しい検索結果が届いたら、開いていたアクションメニューは前提(対象アイテム)が
  // 崩れるため必ず閉じる。デバウンス中にメニューを開いた場合(先にArrowRightで
  // メニューを開いた直後にデバウンスが解決するケース)もここで確実に閉じられる。
  actionMenuOpen = false;
  actionMenuActions = [];
  actionMenuFocusIndex = -1;
  renderResults();
}

function renderResults() {
  resultsList.innerHTML = "";
  emptyMessage.style.display = currentItems.length === 0 ? "block" : "none";

  currentItems.forEach((item, index) => {
    const li = document.createElement("li");
    li.className = index === focusedIndex ? "focused" : "";

    if (item.icon_domain) {
      const iconImg = document.createElement("img");
      iconImg.className = "item-icon";
      iconImg.src = `https://icons.bitwarden.net/${encodeURIComponent(item.icon_domain)}/icon.png`;
      iconImg.alt = "";
      iconImg.onerror = () => {
        const placeholder = document.createElement("span");
        placeholder.className = "item-icon-placeholder";
        iconImg.replaceWith(placeholder);
      };
      li.appendChild(iconImg);
    } else {
      const placeholder = document.createElement("span");
      placeholder.className = "item-icon-placeholder";
      li.appendChild(placeholder);
    }

    const nameSpan = document.createElement("span");
    nameSpan.className = "item-name";
    nameSpan.textContent = item.name;
    li.appendChild(nameSpan);

    const username = item.username;
    if (username) {
      const userSpan = document.createElement("span");
      userSpan.className = "item-username";
      userSpan.textContent = username;
      li.appendChild(userSpan);
    }

    li.appendChild(buildTrailingBlock(item, index));

    li.addEventListener("mouseenter", (event) => {
      // メニュー表示中に他行へフォーカスが移ると、開いているメニューの前提
      // (対象アイテム)が崩れるため無視する(design.md 決定2関連のリスク対策)。
      if (actionMenuOpen) {
        return;
      }
      // scrollIntoView() 等のレイアウト変更による亡霊 mouseenter を無視する(#128)。
      // カーソルが物理的に動いていなければ、発火時の座標は直近の本物の mousemove の
      // 座標と一致する。本物の移動による進入なら、必ずどちらかの座標が変化している。
      if (event.clientX === lastRealMouseX && event.clientY === lastRealMouseY) {
        return;
      }
      if (focusedIndex === index) {
        return;
      }
      const previousIndex = focusedIndex;
      focusedIndex = index;
      updateFocusRows(previousIndex);
    });

    resultsList.appendChild(li);
  });

  if (focusedIndex >= 0) {
    const focusedEl = resultsList.children[focusedIndex];
    if (focusedEl) {
      focusedEl.scrollIntoView({ block: "nearest" });
    }
  }
}

// フォーカス行の末尾ブロック(ショートカットヒント、またはアクションメニュー展開中は
// アクションメニュー)を作る。
function buildTrailingBlock(item, index) {
  if (actionMenuOpen && index === focusedIndex) {
    return renderActionMenu(item);
  }
  const placeholder = document.createElement("span");
  placeholder.className = "row-trailing-placeholder";
  return placeholder;
}

// 矢印キー/マウスホバーによるフォーカス行の移動時に呼ぶ。アイコンを含む行全体を
// 作り直さず、影響を受ける2行(旧フォーカス行・新フォーカス行)の `.focused` クラスと
// 末尾ブロックだけを更新する。行のDOM要素自体(アイコン含む)を作り直さないことで、
// アイコンが移動のたびに再読み込みされてチラつく問題を防ぐ。
// マウスホバーによるフォーカス変更(mouseenter)がスクロール等の亡霊イベントで
// 誤発火しないための座標比較については、行生成時の mouseenter リスナーの
// コメントを参照(#128)。
function updateFocusRows(previousIndex) {
  [previousIndex, focusedIndex].forEach((index) => {
    if (index < 0 || index >= currentItems.length) {
      return;
    }
    const li = resultsList.children[index];
    if (!li) {
      return;
    }
    li.className = index === focusedIndex ? "focused" : "";
    li.lastElementChild.replaceWith(buildTrailingBlock(currentItems[index], index));
  });

  const focusedEl = resultsList.children[focusedIndex];
  if (focusedEl) {
    focusedEl.scrollIntoView({ block: "nearest" });
  }
}

// アクションメニューの開閉・メニュー内フォーカス移動時に呼ぶ。フォーカス行(1行)の
// 末尾ブロックだけを作り直し、他の行(アイコン等)には触れない。
function refreshFocusedRowTrailing() {
  const li = resultsList.children[focusedIndex];
  if (!li) {
    return;
  }
  li.lastElementChild.replaceWith(buildTrailingBlock(currentItems[focusedIndex], focusedIndex));
}

function renderActionMenu(item) {
  const menu = document.createElement("ul");
  menu.className = "action-menu";

  actionMenuActions.forEach((action, index) => {
    const actionLi = document.createElement("li");
    actionLi.className = index === actionMenuFocusIndex ? "focused" : "";

    const labelSpan = document.createElement("span");
    labelSpan.className = "action-label";
    labelSpan.textContent = t(action.labelKey);
    actionLi.appendChild(labelSpan);

    const hintSpan = document.createElement("span");
    hintSpan.className = "action-hint";
    hintSpan.textContent = action.shortcutHint;
    actionLi.appendChild(hintSpan);

    // mousedownの既定動作(フォーカス可能要素ではないためsearchBoxがblurする)を
    // 止め、searchBoxのDOM/ネイティブ側フォーカスを常に維持する。これを怠ると、
    // クリック操作の後にポップアップを閉じて再度開いたときにWebView側の
    // first responderがずれ、キー入力を受け付けなくなることがある。
    actionLi.addEventListener("mousedown", (event) => {
      event.preventDefault();
    });
    actionLi.addEventListener("click", () => {
      executeItemAction(item, action.key);
    });

    menu.appendChild(actionLi);
  });

  return menu;
}

listen("popup-shown", () => {
  // ポップアップウィンドウは非表示/表示を繰り返しても再生成されず(hide()/show()のみ)、
  // JSの状態はそのまま保持され続ける。前回表示時の座標を持ち越すと、今回のカーソル位置とは
  // 無関係な値と比較してしまい mouseenter の亡霊判定(#128)を誤らせるおそれがあるため、
  // 表示のたびに基準座標をリセットする。null にリセットした直後は「一度も本物の
  // mousemove を観測していない」状態になり、design.md に記載の残存リスク(その状態で
  // 発火した mouseenter は無条件で本物の進入として扱われる)と同じ扱いに揃う。
  lastRealMouseX = null;
  lastRealMouseY = null;
  handleShown();
});

listen("popup-hidden", () => {
  hiddenAt = Date.now();
});

// ポップアップ表示中に裏でバックエンド状態が変化した場合(例: トレイメニューからの
// 明示的ロック)、非表示→表示を経由せずに画面を再判定する(design.md参照)。
listen("backend-state-changed", () => {
  syncScreenWithBackend();
});

Promise.all([initI18n(), initPlatform()]).then(() => {
  SHORTCUT_HINTS = formatShortcutForPlatform(t("shortcutHints"));
  footerHints.textContent = SHORTCUT_HINTS;
  updateHelpOverlayKbd();
  handleShown();
});
