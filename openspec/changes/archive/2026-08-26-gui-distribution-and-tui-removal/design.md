## Context

`bw-serve-backend` / `menubar-hotkey-shell` / `quickaccess-search-ui` / `credential-actions-autolock` の実装が完了すると、GUIアプリが現行TUIの主要機能を一通り代替できる状態になる。本changeはv1.0.0の最終ステップとして、TUIコード・配布経路を削除し、GUIアプリ向けの配布・ドキュメントに置き換える。

## Goals / Non-Goals

**Goals:**
- 旧TUIコード(`bin/bw-quickaccess`, `lib/*.sh`, `install.sh`, TUI向けテスト)を削除する。
- README(ja/en)をGUIアプリ前提に全面書き換えする。
- CI/リリースワークフローをGUIアプリのビルド・配布に合わせて更新する。
- セルフビルド + 未署名GitHub Releasesという2本立ての配布手順を整備する。

**Non-Goals:**
- コード署名・notarization。
- Linux対応(v1.1.0)。
- GUIアプリのローカライズ(メッセージ多言語化)。

## Decisions

### 1. 本changeは他の4つのGUI関連change(`bw-serve-backend`/`menubar-hotkey-shell`/`quickaccess-search-ui`/`credential-actions-autolock`)がすべてマージされた後、最後に実施する

TUI削除前にGUIアプリが検索・コピー・ブラウザ起動・自動ロックの一通りの動作確認を終えていることを前提とする。GUIアプリに未検証の欠落機能がある状態でTUIを削除すると、ユーザーが両方使えない空白期間が生まれるため、実施順序を明確にしておく。

### 2. リリース成果物は `tauri build` が生成する `.app` をzip化してGitHub Releasesに添付する

署名なしの `.app` をそのまま配布する。`.github/workflows/release.yml` をmacOS runner上で `tauri build` を実行し、生成物をリリースアセットとして添付する形に置き換える。既存のTUI向けバンドルロジック(単一ファイルへのconcatenation、i18nメッセージファイル同梱)は不要になるため削除する。

### 3. バージョン情報は `tauri.conf.json` / `Cargo.toml` の version フィールドを単一の情報源とする

現行TUIの `bw-quickaccess --version` 相当の確認手段として、GUIアプリのメニュー(コンテキストメニューの「About」相当、または既存の `menubar-presence` capability のコンテキストメニューに項目追加)からバージョンを確認できるようにする。

### 4. CIはmacOS runnerでの `cargo build` / `cargo test` に置き換え、bash向けステップ(`bash -n`, `shellcheck`, `bats`)は削除する

Linux runnerでのCIは、Linux対応(v1.1.0)が始まるまでは追加しない。

### 5. 既存ユーザー向けの移行案内をREADMEに明記する

curlワンライナーで `~/.local/bin/bw-quickaccess` にインストール済みの既存ユーザーは、リポジトリ側の変更だけでは旧TUIが自動的に消えない。README刷新の際に、旧TUIの手動アンインストール手順(`rm ~/.local/bin/bw-quickaccess`)と新GUIアプリへの案内を明記する。

## Risks / Trade-offs

- [TUI削除後にGUIアプリで発見されなかった機能欠落があると、ユーザーが機能低下を経験する] → Decision 1のとおり、他4changeの動作確認を終えてから本changeを実施する運用でリスクを下げる。
- [未署名配布のため、GitHub ReleasesからダウンロードしたユーザーはGatekeeperの「開発元を確認できません」警告に遭遇する] → README上に右クリック→開くでの回避手順を明記する(署名・notarizationは将来のchangeで検討)。
- [既存のcurlインストール済みユーザーが、新しいGUIアプリの存在に気づかないまま古いTUIを使い続ける可能性] → READMEの目立つ位置に移行案内を記載する。

## Migration Plan

- 本changeのマージをもってTUI関連ファイルを削除する(**BREAKING**)。
- ロールバックが必要な場合はgit revertで復元可能(ただしTUI・GUI双方が並存する状態に戻る点に注意)。
- データマイグレーションは発生しない(vaultデータ自体はBitwardenサーバ側にあり、ローカルの認証情報キャッシュ形式が変わるのみ)。

## Open Questions

- GitHub Releasesのアセット命名規則・zip構成は実装時に決定する。
- README刷新の具体的な章立ては実装時に詰める(既存README.md/README.ja.mdの構成を参考にする)。
