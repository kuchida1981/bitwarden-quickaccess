## ADDED Requirements

### Requirement: session tokenのOSキーチェーンへのキャッシュ
システムは、`bw unlock` で取得した session token を OS のキーチェーン(macOS Keychain、または Linux の Secret Service)に保存し、以後の実行ではキャッシュされた token を再利用しなければならない(SHALL)。これにより、TTL 内であればマスターパスワードの再入力を求めてはならない(SHALL NOT)。

#### Scenario: 初回起動時はマスターパスワードを要求しキャッシュを作成する
- **WHEN** キャッシュされた session token が存在しない状態でツールを起動する
- **THEN** システムはマスターパスワードの入力を求めて `bw unlock` を実行し、取得した session token を OS キーチェーンに保存する

#### Scenario: 有効なキャッシュがあれば再認証を求めない
- **WHEN** 有効期限内の session token が OS キーチェーンにキャッシュされている状態でツールを起動する
- **THEN** システムはマスターパスワードの入力を求めず、キャッシュされた token を使用して vault 操作を行う

### Requirement: TTLによるセッション有効期限管理
システムは、session token の発行時刻を記録し、設定された TTL(既定値を持ち、上書き可能)を超過した場合はキャッシュを無効とみなし、再度 `bw unlock` によるマスターパスワード入力を要求しなければならない(SHALL)。

#### Scenario: TTL超過後は再認証を要求する
- **WHEN** キャッシュされた session token の発行時刻からの経過時間が TTL を超えている状態でツールを起動する
- **THEN** システムはキャッシュを使用せず、マスターパスワードの入力を求めて再度 `bw unlock` を実行する

### Requirement: セッション無効時の実利用時フォールバック
システムは、TTL内であっても `bw` コマンドの実行結果からキャッシュ済み session token が無効(ロック済み等)と判明した場合、その場で再度 `bw unlock` によるマスターパスワード入力を要求しなければならない(SHALL)。

#### Scenario: bwコマンドがsession無効を返した場合に再認証する
- **WHEN** キャッシュされた session token を用いた `bw` コマンドの実行が、session が無効であることを示すエラーを返す
- **THEN** システムはマスターパスワードの入力を求めて再度 `bw unlock` を実行し、キャッシュを更新する

### Requirement: session tokenをコマンドライン引数として渡さない
システムは、session token を `bw` コマンドへ渡す際、プロセスの引数一覧から読み取り可能な形(コマンドライン引数)を用いてはならず(SHALL NOT)、環境変数経由で渡さなければならない(SHALL)。

#### Scenario: session tokenが環境変数経由で渡される
- **WHEN** システムが session token を使って `bw` コマンドを実行する
- **THEN** session token は環境変数を介して渡され、コマンドライン引数には出現しない
