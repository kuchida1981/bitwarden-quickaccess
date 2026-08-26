## REMOVED Requirements

### Requirement: curl ワンライナーによるインストール
**Reason**: `install.sh` によるシェルスクリプト配布はTUI廃止に伴い廃止される。
**Migration**: GUIアプリは `gui-distribution` capability(セルフビルド + 未署名GitHub Releases)に置き換わる。

### Requirement: デフォルトはユーザー権限インストール
**Reason**: `install.sh` 固有のインストール先制御はTUI廃止に伴い廃止される。
**Migration**: GUIアプリは `.app` バンドルとして `/Applications` 等ユーザーが選んだ場所に配置する一般的なmacOSアプリの慣習に従う。

### Requirement: --prefix オプションによるインストール先変更
**Reason**: `install.sh` の `--prefix` オプションはTUI廃止に伴い廃止される。
**Migration**: 該当なし。

### Requirement: バージョン解決
**Reason**: `install.sh --version` によるバージョン指定インストールはTUI廃止に伴い廃止される。
**Migration**: GUIアプリはGitHub Releasesの各バージョンから直接ダウンロードする形になる(`gui-distribution` capability)。

### Requirement: PATH 未設定時の警告表示
**Reason**: PATH経由でのコマンド実行を前提としたTUI固有の警告表示はGUIアプリには適用されない。
**Migration**: 該当なし。

### Requirement: install.sh 自体の依存
**Reason**: `install.sh` 自体がTUI廃止に伴い削除される。
**Migration**: 該当なし。

### Requirement: install.sh の再実行によるアップデート
**Reason**: `install.sh` 再実行によるアップデート方式はTUI廃止に伴い廃止される。
**Migration**: GUIアプリのアップデートはGitHub Releasesから新バージョンを再ダウンロードする形になる(`gui-distribution` capability)。
