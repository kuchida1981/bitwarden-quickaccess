# bw-quickaccess

Read this in English / [日本語版はこちら](README.ja.md)

A terminal quick-access tool equivalent to 1Password Quick Access, built on top of `bw` (Bitwarden CLI), `jq`, and `fzf`. It lets you incrementally search vault items and copy the username, password, or TOTP to the clipboard.

## Requirements

- macOS, or Linux with a desktop GUI environment (with GNOME Keyring / KWallet or similar running)
- [`bw` (Bitwarden CLI)](https://bitwarden.com/help/cli/) — must already be logged in via `bw login`
- `jq`
- `fzf` (0.73.0 or later)
- A clipboard copy command
  - macOS: `pbcopy` (built in)
  - Linux (Wayland): `wl-copy`
  - Linux (X11): `xclip` or `xsel`
- An OS keychain integration command (used to cache the session token)
  - macOS: `security` (built in)
  - Linux: `secret-tool` (from the `libsecret-tools` package)

Depending on how you install it, you'll also need:
- **Using the installer (install.sh)**
  - `curl` (git is not required)
- **Cloning the source and running it directly**
  - `git`

If any required tool is missing, the tool prints installation instructions and exits with an error at startup.

The search screen and field-selection screen run fullscreen (using the terminal's alternate screen buffer). Your terminal scrollback is temporarily hidden while either screen is open, and the original screen content is restored when you exit.

## Installation

You can install it easily by running the following command.

```sh
curl -fsSL https://raw.githubusercontent.com/kuchida1981/bitwarden-quickaccess/main/install.sh | bash
```

By default, it installs to `~/.local/bin/bw-quickaccess` under your user account (no elevated privileges required).

### Options

To customize the installation, pass options like this:

- **Change the install location (`--prefix`)**
  To change the default install location, use the `--prefix` option.
  ```sh
  curl -fsSL https://raw.githubusercontent.com/kuchida1981/bitwarden-quickaccess/main/install.sh | bash -s -- --prefix /opt/bwqa
  ```
  In this example, it installs to `/opt/bwqa/bin/bw-quickaccess`.

- **Install a specific version (`--version`)**
  To install a specific version other than the latest, use the `--version` option.
  ```sh
  curl -fsSL https://raw.githubusercontent.com/kuchida1981/bitwarden-quickaccess/main/install.sh | bash -s -- --version v0.1.0
  ```

### Updating

To update, re-run the same curl command used for installation. Re-running it shows an update message from the old version to the new one.

You can check the currently installed version with:

```sh
bw-quickaccess --version
```
Or, if the install location isn't on your PATH, run it directly:
```sh
~/.local/bin/bw-quickaccess --version
```

### Uninstalling

To remove `bw-quickaccess`, delete the executable.

```sh
rm ~/.local/bin/bw-quickaccess
```

If you changed the install location with `--prefix`, remove it like this instead:

```sh
rm <prefix>/bin/bw-quickaccess
```

## Usage

If installed:
```sh
bw-quickaccess
```
Note: the install location (e.g. `~/.local/bin`) needs to be on your PATH. If it isn't, run it with the full path, e.g. `~/.local/bin/bw-quickaccess`.

If running directly from a cloned source checkout (e.g. for development):
```sh
bin/bw-quickaccess
```

1. Incrementally search vault items on the search screen (fzf)
   - `Enter`: select an item and move to the field selection screen
   - `ctrl-r`: copy the password of the filtered item directly (stays on this screen)
   - `ctrl-o`: copy the username of the filtered item directly (stays on this screen)
   - `ctrl-t`: copy the TOTP of the filtered item directly (stays on this screen)
2. On the field selection screen, choose the field you want to copy
   - `Enter`: copy the selected row
   - `ctrl-r`: copy the password directly
   - `ctrl-o`: copy the username directly
   - `ctrl-t`: copy the TOTP directly
   - The screen stays open after copying, so you can copy other fields of the same item in succession
   - `Esc`: go back to the search screen
   - `q`: quit the tool
3. On the next launch, it starts from the field selection screen for the last selected item (skipping the search). Press `Esc` to go back to the search screen if you want to look up a different item.

### About sessions (login state)

On first run, you'll be prompted for your master password via `bw unlock`. The resulting session token is cached in the OS keychain, and you won't be prompted again within the default 15-minute window (configurable via the `BWQA_SESSION_TTL_SECONDS` environment variable).

To discard the cached session:

If installed:
```sh
bw-quickaccess lock
```

If running directly from source:
```sh
bin/bw-quickaccess lock
```

### Display language

CLI messages are automatically selected from the `LANG`/`LC_ALL` environment variables between Japanese and English (anything not starting with `ja` falls back to English). You can override this explicitly with the `BWQA_LANG` environment variable (`ja` or `en`):

```sh
BWQA_LANG=en bw-quickaccess
```

### Out of scope

- Deep-link integration with the Bitwarden desktop app (the app doesn't support navigating directly to a specific item)
- Automatically clearing the clipboard
- Support for headless/SSH-only Linux environments

## For developers: running tests

Unit tests for `lib/*.sh` are written with [bats-core](https://github.com/bats-core/bats-core). Static analysis uses [shellcheck](https://www.shellcheck.net/) (see `.shellcheckrc` at the repository root for excluded rules).

### Setup

```sh
# macOS
brew install bats-core shellcheck

# Linux (Debian/Ubuntu family)
sudo apt-get install -y bats shellcheck
```

### Running

```sh
# Syntax check
bash -n bin/bw-quickaccess
for f in lib/*.sh; do bash -n "$f"; done

# Static analysis (production code uses -x for cross-file analysis; test code is analyzed standalone)
shellcheck -x bin/bw-quickaccess
shellcheck test/helpers/*.bash test/lib/*.bats

# Unit tests
bats test/lib/*.bats
```

GitHub Actions (`.github/workflows/ci.yml`) runs the same checks automatically on both `macos-latest` and `ubuntu-latest` for every push and pull request.

See `openspec/changes/add-quickaccess-cli/` for detailed requirements and design.
