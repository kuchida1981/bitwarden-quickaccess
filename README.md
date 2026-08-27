# bw-quickaccess

Read this in English / [日本語版はこちら](README.ja.md)

A menu-bar quick-access app for Bitwarden, equivalent to 1Password Quick Access, built on top of `bw` (Bitwarden CLI) via `bw serve` and [Tauri](https://tauri.app/). Press a global hotkey from anywhere to search your vault and copy the username, password, or TOTP to the clipboard — or open the item's URL in your browser. Copied values are automatically cleared from the clipboard after 30 seconds (or immediately when the vault is locked), unless you've already copied something else in the meantime.

> **Coming from the old terminal (TUI) version?** See [Migrating from the old TUI](#migrating-from-the-old-tui) below.

## Requirements

- macOS (Linux support is planned for a future release; not yet supported)
- [`bw` (Bitwarden CLI)](https://bitwarden.com/help/cli/) — must already be logged in via `bw login` (the vault can be locked; the app handles unlocking)

To **self-build**, you additionally need:
- [Rust toolchain](https://www.rust-lang.org/tools/install) (stable, via `rustup`)
- [Tauri CLI](https://v2.tauri.app/reference/cli/): `cargo install tauri-cli --locked`

## Install

### Option 1: Homebrew

```bash
brew tap kuchida1981/bitwarden-quickaccess
brew install --cask bw-quickaccess
```

This build targets **Apple Silicon (arm64) only**; Intel Macs are not supported. On Homebrew 6 and later, the first time you use a third-party tap you may be prompted to trust it:

```bash
brew trust --tap kuchida1981/bitwarden-quickaccess
```

The app is **not code-signed or notarized**, so installing via Homebrew does not bypass Gatekeeper. On first launch you'll still need to either right-click the app and choose **Open** (see Option 2, step 3), or reinstall with `brew install --cask --no-quarantine bw-quickaccess`.

### Option 2: Download from GitHub Releases

1. Download `Bitwarden Quick Access_aarch64.app.tar.gz` from the [Releases page](https://github.com/kuchida1981/bitwarden-quickaccess/releases). This build targets **Apple Silicon (arm64) only**; Intel Macs are not currently supported by the prebuilt release (self-build from source instead — see Option 3).
2. Extract it (double-click in Finder, or `tar -xzf "Bitwarden Quick Access_aarch64.app.tar.gz"`) and move `Bitwarden Quick Access.app` to `/Applications` (or anywhere you like).
3. The app is **not code-signed or notarized**. On first launch, macOS Gatekeeper will refuse to open it with an "unidentified developer" warning. To open it anyway:
   - Right-click (or Control-click) `Bitwarden Quick Access.app` in Finder and choose **Open**, then confirm **Open** in the dialog that appears.
   - You only need to do this once; subsequent launches work normally.

### Option 3: Self-build from source

```bash
git clone https://github.com/kuchida1981/bitwarden-quickaccess.git
cd bitwarden-quickaccess/app/src-tauri
cargo tauri build
```

The built `.app` is placed under `target/release/bundle/macos/Bitwarden Quick Access.app`. Move it to `/Applications` if you like.

For local development (runs the app without producing a distributable bundle):

```bash
cd app/src-tauri
cargo run
```

## Usage

1. Launch `Bitwarden Quick Access.app`. It has no Dock icon or window on startup — look for its icon in the menu bar.
2. Press **⇧⌘Space** (Shift+Cmd+Space) from anywhere to toggle the popup.
3. If the vault is locked, enter your master password to unlock.
4. Type to search incrementally. Use the **↑ / ↓** arrow keys to move the highlighted selection.
5. With an item highlighted, use one of these shortcuts:

   | Shortcut | Action |
   |---|---|
   | `⌘C` | Copy username |
   | `⌘⇧C` | Copy password |
   | `⌥⌘C` | Copy TOTP code |
   | `Enter` | Open the item's URL in your default browser |

   The popup closes automatically after the action completes.
6. The popup also closes automatically when it loses focus (e.g. clicking elsewhere).

### Menu bar icon

Click the tray icon to see the current lock status, whether the global hotkey registered successfully, toggle **launch at login**, check the installed version, or quit the app.

### Auto-lock

The vault automatically re-locks after 15 minutes of inactivity (no search, copy, or browser-open actions). This mirrors the previous TUI's session TTL behavior. There is currently no UI to change this timeout.

### Language

The UI text (menu bar, popup) follows your macOS system language: Japanese if your system language is Japanese, English otherwise. There is no in-app language switcher; changing the system language requires an app restart to take effect.

## Migrating from the old TUI

The previous terminal-based tool (`bin/bw-quickaccess`, installed via `install.sh`) has been removed from this repository as of this GUI rewrite. If you previously installed it via the `curl` one-liner, it is **not automatically removed** — delete it manually:

```bash
rm "$HOME/.local/bin/bw-quickaccess"
```

(If you installed with a custom `--prefix`, adjust the path accordingly: `$PREFIX/bin/bw-quickaccess`.)

Then install the new GUI app using one of the options above.

## Out of scope

- Linux support (planned for a future release)
- Code signing / notarization
- In-app language switcher (the UI language follows the macOS system language; see [Language](#language))
- Configurable idle-lock timeout or hotkey remapping
