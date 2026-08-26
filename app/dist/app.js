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
const helpOverlay = document.getElementById("help-overlay");

const SEARCH_DEBOUNCE_MS = 150;
let SHORTCUT_HINTS = "";

let currentItems = [];
let focusedIndex = -1;
let debounceTimer = null;
let searchRequestId = 0;
let lastKnownScreen = "unlock";

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

async function handleShown() {
  actionMenuOpen = false;
  actionMenuActions = [];
  actionMenuFocusIndex = -1;
  helpOpen = false;
  helpOverlay.classList.remove("visible");
  showScreen(lastKnownScreen);
  if (lastKnownScreen === "search") {
    searchBox.focus();
  } else if (lastKnownScreen === "unlock") {
    passwordInput.value = "";
    unlockError.textContent = "";
    passwordInput.focus();
  }

  let lockState = "disconnected";
  try {
    lockState = await invoke("get_lock_state");
  } catch {
    // 取得に失敗した場合はアンロックフォーム側にフォールバックする
  }

  const actualScreen = lockState === "unlocked" ? "search" : lockState === "locked" ? "unlock" : "error";

  if (actualScreen === lastKnownScreen) {
    if (actualScreen === "search") {
      searchBox.value = "";
      await runSearch("");
    } else if (actualScreen === "error") {
      try {
        const err = await invoke("get_backend_error");
        errorMessage.textContent = err || "";
      } catch {
        errorMessage.textContent = "";
      }
    }
  } else {
    showScreen(actualScreen);
    if (actualScreen === "search") {
      searchBox.value = "";
      searchBox.focus();
      await runSearch("");
    } else if (actualScreen === "unlock") {
      passwordInput.value = "";
      unlockError.textContent = "";
      passwordInput.focus();
    } else if (actualScreen === "error") {
      try {
        const err = await invoke("get_backend_error");
        errorMessage.textContent = err || "";
      } catch {
        errorMessage.textContent = "";
      }
    }
  }

  lastKnownScreen = actualScreen;
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
    showScreen("search");
    lastKnownScreen = "search";
    searchBox.value = "";
    searchBox.focus();
    await runSearch("");
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

// event.key ではなく event.code(物理キー位置)で判定する(handleActionShortcutの
// ⌘Cと同じ理由)。ヘルプの開閉どちらからも参照するため共通関数にしておく。
function isHelpToggleShortcut(event) {
  return event.metaKey && event.code === "Slash" && !event.shiftKey && !event.altKey;
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

function moveFocus(delta) {
  if (currentItems.length === 0) {
    return;
  }
  focusedIndex = Math.min(Math.max(focusedIndex + delta, 0), currentItems.length - 1);
  renderResults();
}

// 選択中アイテムの実行可能アクション一覧を組み立てる(design.md 決定4)。
// フィールドを持たない項目は結果配列から除外する(#52)。
function buildActionsForItem(item) {
  const actions = [
    { key: "username", labelKey: "actionCopyUsername", shortcutHint: "⌘C", enabled: !!item.username },
    { key: "password", labelKey: "actionCopyPassword", shortcutHint: "⌘⇧C", enabled: item.has_password },
    { key: "totp", labelKey: "actionCopyTotp", shortcutHint: "⌥⌘C", enabled: item.has_totp },
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
  renderResults();
}

function closeActionMenu() {
  actionMenuOpen = false;
  actionMenuActions = [];
  actionMenuFocusIndex = -1;
  renderResults();
}

// アクションメニュー展開中のキー操作(design.md 決定3)。
// ⌘C系のダイレクトショートカットはメニュー表示中も従来通り動作させ、
// それ以外の未処理キー(検索文字入力等)はメニューの前提が崩れないよう無視する。
function handleActionMenuKeydown(event) {
  if (event.key === "ArrowDown") {
    event.preventDefault();
    actionMenuFocusIndex = Math.min(actionMenuFocusIndex + 1, actionMenuActions.length - 1);
    renderResults();
    return;
  }
  if (event.key === "ArrowUp") {
    event.preventDefault();
    actionMenuFocusIndex = Math.max(actionMenuFocusIndex - 1, 0);
    renderResults();
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
  if (!event.metaKey || event.code !== "KeyC") {
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

    if (actionMenuOpen && index === focusedIndex) {
      li.appendChild(renderActionMenu(item));
    } else {
      const hints = document.createElement("div");
      hints.className = "hints";
      hints.textContent = SHORTCUT_HINTS;
      li.appendChild(hints);
    }

    li.addEventListener("mouseenter", () => {
      // メニュー表示中に他行へフォーカスが移ると、開いているメニューの前提
      // (対象アイテム)が崩れるため無視する(design.md 決定2関連のリスク対策)。
      if (actionMenuOpen) {
        return;
      }
      focusedIndex = index;
      renderResults();
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
  handleShown();
});

initI18n().then(() => {
  SHORTCUT_HINTS = t("shortcutHints");
  handleShown();
});
