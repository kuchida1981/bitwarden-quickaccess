# bw-quickaccess: message strings (English)
# This file is sourced automatically from lib/common.sh. Do not source it directly.

BWQA_MSG_ERR_PREFIX="Error: %s"

# lib/preflight.sh
BWQA_MSG_PREFLIGHT_CMD_NOT_FOUND="Required command '%s' not found. %s"
BWQA_MSG_PREFLIGHT_BW_INSTALL_HINT="Please see https://bitwarden.com/help/cli/ for installation instructions (e.g. brew install bitwarden-cli)."
BWQA_MSG_PREFLIGHT_JQ_INSTALL_HINT="Please install it with 'brew install jq' or your distro's package manager."
BWQA_MSG_PREFLIGHT_FZF_INSTALL_HINT="Please install it with 'brew install fzf' or your distro's package manager."
BWQA_MSG_PREFLIGHT_FZF_VERSION_UNKNOWN="Could not determine the fzf version. Please install fzf %s or later (e.g. brew install fzf)."
BWQA_MSG_PREFLIGHT_FZF_VERSION_TOO_OLD="fzf version is too old (detected: %s / required: %s or later). Please upgrade with 'brew upgrade fzf' or similar."
BWQA_MSG_PREFLIGHT_OS_UNSUPPORTED="Unsupported OS (%s). Only macOS or Linux (desktop environment) is supported."
BWQA_MSG_PREFLIGHT_DISPLAY_NOT_FOUND="Could not detect a Wayland/X11 display. Please run this in a desktop GUI environment (headless/SSH-only environments are not supported)."
BWQA_MSG_PREFLIGHT_MACOS_BUILTIN_HINT="This should be built into macOS by default. Please check your PATH."
BWQA_MSG_PREFLIGHT_WL_COPY_NOT_FOUND="wl-copy not found. Please install it with 'apt install wl-clipboard' or similar."
BWQA_MSG_PREFLIGHT_XCLIP_NOT_FOUND="Neither xclip nor xsel was found. Please install one with 'apt install xclip' or similar."
BWQA_MSG_PREFLIGHT_SECRET_TOOL_NOT_FOUND="secret-tool not found. Please install it with 'apt install libsecret-tools' or similar."
BWQA_MSG_PREFLIGHT_KEYRING_SELFTEST_FAILED="Warning: failed to connect to the keyring backend (GNOME Keyring/KWallet, etc.). Session caching is disabled; you will be prompted for the master password every time."

# lib/session.sh
BWQA_MSG_SESSION_UNLOCKING="Unlocking the vault..."
BWQA_MSG_SESSION_UNLOCK_FAILED="bw unlock failed. Please check your master password."
BWQA_MSG_SESSION_EMPTY="bw unlock returned an empty session."
BWQA_MSG_SESSION_REAUTH="The session is no longer valid; re-authenticating."
BWQA_MSG_SESSION_BW_CMD_FAILED="bw %s failed: %s"
BWQA_MSG_SESSION_CACHE_CLEARED="Session cache cleared."

# lib/search.sh
BWQA_MSG_SEARCH_LOADING_ITEMS="Loading the list of vault items..."
BWQA_MSG_SEARCH_FETCH_FAILED="Failed to retrieve vault items."
BWQA_MSG_SEARCH_FZF_HEADER="Enter: select item  ctrl-o: copy username  ctrl-r: copy password  ctrl-t: copy TOTP directly  Esc: quit"

# lib/fields.sh
BWQA_MSG_FIELDS_LOADING_ITEM="Retrieving item information..."
BWQA_MSG_FIELDS_ITEM_FETCH_FAILED="Failed to retrieve item information."
BWQA_MSG_FIELDS_NO_COPYABLE_FIELDS="No copyable fields for this item: %s"
BWQA_MSG_FIELDS_ROW_USERNAME="Copy username (ctrl-o)"
BWQA_MSG_FIELDS_ROW_PASSWORD="Copy password (ctrl-r)"
BWQA_MSG_FIELDS_ROW_TOTP="Copy TOTP (ctrl-t)"
BWQA_MSG_FIELDS_FZF_HEADER="Enter: copy selected field  ctrl-r: password  ctrl-o: username  ctrl-t: totp  Esc: back to search  q: quit"
BWQA_MSG_FIELDS_LABEL_USERNAME="username"
BWQA_MSG_FIELDS_LABEL_PASSWORD="password"
BWQA_MSG_FIELDS_LABEL_TOTP="TOTP"
BWQA_MSG_FIELDS_COPY_FAILED="Copy failed"
BWQA_MSG_FIELDS_VALUE_NOT_SET="%s is not set"
BWQA_MSG_FIELDS_COPY_SUCCESS="%s copied"

# bin/bw-quickaccess
BWQA_MSG_USAGE_TEXT="Usage: bw-quickaccess [lock]

  (no argument)   Search vault items and copy a field to the clipboard
  lock            Discard the cached session (prompts for the master password again next time)
  -v, --version   Show the version
  -h, --help      Show this help"
BWQA_MSG_UNKNOWN_ARG="Unknown argument: %s"
