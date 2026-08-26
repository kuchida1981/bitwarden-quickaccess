## ADDED Requirements

### Requirement: セルフビルドによるインストール
ユーザーは、リポジトリを `git clone` し `tauri build` を実行することで、macOS向けGUIアプリをビルドして利用できなければならない(SHALL)。

#### Scenario: ソースからビルドする
- **WHEN** ユーザーがリポジトリをcloneし `tauri build` 相当のコマンドを実行する
- **THEN** macOS向けの `.app` バンドルが生成される

### Requirement: 未署名GitHub Releasesによる配布
プロジェクトは、タグ付きリリースごとに未署名の `.app` をビルドし、GitHub Releasesのアセットとして添付しなければならない(SHALL)。

#### Scenario: リリースタグ作成でアセットが添付される
- **WHEN** リポジトリにリリース用のタグがpushされる
- **THEN** CIがmacOS向け `.app` をビルドし、そのリリースのアセットとして自動的に添付する

### Requirement: Gatekeeper警告時の回避手順の明記
README は、未署名アプリをダウンロードした際にGatekeeperの「開発元を確認できません」警告に遭遇した場合の回避手順(右クリック→開く)を明記しなければならない(SHALL)。

#### Scenario: READMEに回避手順が記載されている
- **WHEN** ユーザーがGitHub ReleasesからダウンロードしたアプリをGatekeeperにブロックされる
- **THEN** README記載の手順(右クリック→開く)に従うことで起動できる

### Requirement: 旧TUIからの移行案内
README は、既存のcurlインストールによる旧TUI(`bin/bw-quickaccess`)からGUIアプリへ移行するユーザー向けに、旧TUIのアンインストール手順を明記しなければならない(SHALL)。

#### Scenario: 旧TUIのアンインストール手順が記載されている
- **WHEN** 旧TUIをcurlワンライナーでインストール済みのユーザーがREADMEを参照する
- **THEN** `~/.local/bin/bw-quickaccess` 等の旧TUI実行ファイルを削除する手順が案内されている
