const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const unlockScreen = document.getElementById("unlock-screen");
const searchScreen = document.getElementById("search-screen");
const unlockForm = document.getElementById("unlock-form");
const passwordInput = document.getElementById("master-password");
const unlockError = document.getElementById("unlock-error");
const unlockButton = unlockForm.querySelector("button");
const searchBox = document.getElementById("search-box");
const resultsList = document.getElementById("results");
const emptyMessage = document.getElementById("empty-message");

const SEARCH_DEBOUNCE_MS = 150;
const SHORTCUT_HINTS = "⌘C ユーザー名 / ⌘⇧C パスワード / ⌥⌘C TOTP / Enter ブラウザで開く";

let currentItems = [];
let focusedIndex = -1;
let debounceTimer = null;
let searchRequestId = 0;
let lastKnownScreen = "unlock";

function showScreen(name) {
  unlockScreen.classList.toggle("active", name === "unlock");
  searchScreen.classList.toggle("active", name === "search");
}

async function handleShown() {
  showScreen(lastKnownScreen);
  if (lastKnownScreen === "search") {
    searchBox.focus();
  } else {
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

  const actualScreen = lockState === "unlocked" ? "search" : "unlock";

  if (actualScreen === lastKnownScreen) {
    if (actualScreen === "search") {
      searchBox.value = "";
      await runSearch("");
    }
  } else {
    showScreen(actualScreen);
    if (actualScreen === "search") {
      searchBox.value = "";
      searchBox.focus();
      await runSearch("");
    } else {
      passwordInput.value = "";
      unlockError.textContent = "";
      passwordInput.focus();
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
    unlockError.textContent = typeof err === "string" ? err : "アンロックに失敗しました。";
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

searchBox.addEventListener("keydown", (event) => {
  if (event.key === "ArrowDown") {
    event.preventDefault();
    moveFocus(1);
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    moveFocus(-1);
  }
});

function moveFocus(delta) {
  if (currentItems.length === 0) {
    return;
  }
  focusedIndex = Math.min(Math.max(focusedIndex + delta, 0), currentItems.length - 1);
  renderResults();
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

    const username = item.login && item.login.username;
    if (username) {
      const userSpan = document.createElement("span");
      userSpan.className = "item-username";
      userSpan.textContent = username;
      li.appendChild(userSpan);
    }

    const hints = document.createElement("div");
    hints.className = "hints";
    hints.textContent = SHORTCUT_HINTS;
    li.appendChild(hints);

    li.addEventListener("mouseenter", () => {
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

listen("popup-shown", () => {
  handleShown();
});

handleShown();
