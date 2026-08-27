## Why

Finder上のアプリ名が `bw-quickaccess` のままで無機質だという指摘（Issue #68 のコメント）を受け、PR #107（`rename-app-display-name`）ではアプリ内のウィンドウタイトル・アンロック画面の見出しのみを親しみやすい表示名に変更した。その際、`productName`（Finder上のアプリ名の元）は Homebrew 配布・自動起動への影響範囲が未整理だったため意図的に据え置いた。Issue #108 では、その影響範囲を洗い出した上で `productName` 自体の変更に踏み込むことを求めている。今回、影響範囲（Homebrew Cask、ログイン時自動起動の LaunchAgent 登録、CI のリリースアセット名）を調査し、現在のユーザーが開発者本人のみ（検証中）という前提のもとで、恒久的な自動化コストをかけずに安全に変更できる方針を固めた。

## What Changes

- `app/src-tauri/tauri.conf.json` の `productName` を `"bw-quickaccess"` から `"Bitwarden Quick Access"` に変更する。これにより Finder 上のアプリ名・`.app` バンドル名・リリースアセット名（`.app.tar.gz`）がこの新しい名称ベースになる。
- `identifier`（`com.kuchida1981.bw-quickaccess`）および `Cargo.toml` のパッケージ名・バイナリ名（`bw-quickaccess-gui`）は変更しない。既存ユーザーのアクセシビリティ権限・ビルド成果物への影響を避けるため、明示的に対象外とする。
- README.md / README.ja.md 内の `bw-quickaccess.app` / `bw-quickaccess_aarch64.app.tar.gz` といったファイル名の記載を、新しい `productName` に基づく名称に更新する。
- Homebrew tap リポジトリ（`kuchida1981/homebrew-bitwarden-quickaccess`、別リポジトリ）の Cask 定義（`Casks/bw-quickaccess.rb` の `app` / `url` / `caveats` 各行）を、新しいアセット名に合わせて手動で1回更新する運用を CONTRIBUTING.md のリリース手順に注記として追加する。`brew bump-cask-pr` による自動更新は `version`/`sha256` のみが対象であり、ファイル名自体は追従しないため。
- ログイン時自動起動（`tauri_plugin_autostart` の LaunchAgent 登録）のファイル名が `productName` に依存するため、既存インストールでは1回だけ自動起動のオン/オフ表示がリセットされうることを design.md に既知の制約として記録する。コード上の移行対応は行わない（現状の利用者が開発者本人のみのため、恒久対応の優先度は低いと判断）。

## Capabilities

### New Capabilities
- `app-bundle-naming`: Finder上のアプリ表示名（`productName`）と、変更しない識別子群（`identifier`、Cargoパッケージ名）の関係を定義する。

### Modified Capabilities
(なし。既存の `gui-distribution` / `homebrew-distribution` / `homebrew-tap-release-automation` / `login-item-autostart` の SHALL要件はいずれも変更しない。Cask定義やREADMEのファイル名記載は実装詳細であり、既存スペックが定めるユーザー向け振る舞い自体は変わらない。)

## Impact

- `app/src-tauri/tauri.conf.json`: `productName` の変更
- `README.md` / `README.ja.md`: `.app` / `.app.tar.gz` のファイル名記載の更新
- `CONTRIBUTING.md`: リリース手順に、このリリース限定の Cask 手動更新手順を注記
- 別リポジトリ `kuchida1981/homebrew-bitwarden-quickaccess`（`Casks/bw-quickaccess.rb`）: 手動更新が必要（本リポジトリのコード変更対象外）
- 既存インストール済みユーザー（現状は開発者本人のみ）: アップデート後、ログイン時自動起動の表示が一度リセットされる可能性がある
