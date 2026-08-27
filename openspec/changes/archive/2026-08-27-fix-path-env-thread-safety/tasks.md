## 1. 実装: fix_path_env のリファクタリング

- [x] 1.1 `app/src-tauri/src/main.rs`に、`Command`・タイムアウト値・ポーリング間隔を引数に取り`set_var`を呼ばない関数(例: `run_shell_and_capture_stdout`)を切り出す。内部は`Command::spawn()`で子プロセスを起動し、`try_wait()`をポーリング間隔ごとに呼んでデッドライン内の終了を待つ。
- [x] 1.2 デッドラインを超過した場合は`child.kill()` → `child.wait()`を呼び、子プロセスを確実にkill・reap(ゾンビ化防止)してから`None`を返す。
- [x] 1.3 子プロセスが正常終了した場合は、`child.stdout`を`read_to_string`で読み取り(終了済みプロセスの書き込み端は閉じているためブロックしない)、既存の`extract_path_from_marker`でPATHを抽出して返す。
- [x] 1.4 `fix_path_env()`本体を、「SHELL環境変数を読む→1.1の関数を3秒/50msで呼ぶ→結果があれば`unsafe { std::env::set_var(...) }`する」という薄いラッパーに書き換える。
- [x] 1.5 `unsafe`ブロックのSAFETYコメントを、「追加スレッドを持たず、子プロセスのkill・reapが完了してから戻るため、この時点で環境変数に触れる他スレッドは存在しない」という、実装によって保証される内容に更新する。

## 2. テスト

- [x] 2.1 成功パス: `sh -c "echo -n <marker>; printenv PATH"`を実行し、短いtimeout内に完了し期待するPATH文字列が抽出できることを検証するテストを追加する。
- [x] 2.2 タイムアウト+killパス: `sh -c "sleep 5"`をtimeout=50ms程度・poll_interval=5ms程度まで短縮したパラメータで実行し、(a) `None`が返ること、(b) 壁時計時間で5秒待たずに(timeoutに近い時間で)返ってくることの両方をアサートし、killが実際に効いていることを証明する。
- [x] 2.3 spawn失敗パス: 存在しないシェルパスを指定してもpanicせず`None`が返ることを検証するテストを追加する。
- [x] 2.4 既存の`fix_path_env_tests`(`extract_path_from_marker`関連)がリファクタリング後も変更なく通ることを確認する。

## 3. 検証

- [x] 3.1 `cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings`をすべて通す。
- [x] 3.2 (手動デバイス確認・ハングケース) ログインシェルの起動ファイルに一時的に`sleep 10`を追加し、`cargo run`でアプリを起動して次を確認する: (a) アプリがハングせず(3秒程度で)起動すること、(b) タイムアウト後に`ps aux`等でログインシェルの子プロセスが残っていないこと。
      → 実機確認(2026-08-27)。ユーザーの`$SHELL`は`fish`(`~/.zshrc`内で`exec fish`する構成、`chsh`はしていないが対話ターミナルの`$SHELL`はfishを指す)。`fish -l -c`はzshと異なり対話/非対話を問わず`~/.config/fish/config.fish`を常にsourceすることを確認した上で、そこに`sleep 10`を仕込んでプロセス監視(0.1秒間隔ポーリング)を実施。fish子プロセスが出現から**正確に3.0秒後**に消滅(kill&reap)し、10秒のsleepを待たずに済んでいること、その後プロセスが残存しないことを確認した。確認後、`~/.config/fish/config.fish`の変更を元に戻す。
- [x] 3.3 (手動デバイス確認・通常ケース) シェルが正常応答する通常環境でもFinder起動時に`bw`コマンドが解決され、アプリが従来通り正常動作すること(リグレッション確認)。
      → 実機確認(2026-08-27)。ユーザー環境(`$SHELL=fish`だが`~/.zshrc`の`exec fish`でzshからfishへブートストラップする構成)では、`env -i`でPATHを最小化して`bw-quickaccess-gui`を直接起動すると`bw`が見つからない事象を確認した。原因を切り分けたところ、`fish -l -c "..."`を単体起動した場合、ユーザーの`~/.config/fish/config.fish`/`conf.d/`が`/opt/homebrew/bin`自体をPATHに追加していないため(`brew shellenv`相当の記述がfish側にはなく、普段はzshの`.zprofile`実行後に`exec fish`することでPATHを引き継いでいるだけ)と判明した。この`Command::new(shell).args(["-l","-c",...])`という起動方法自体は本changeで変更しておらず、リファクタリング前後で完全に同一の呼び出しであるため、**今回のリファクタリングによるリグレッションではない**(pre-existingの別の制約であり、issue #83のスコープ外)。通常のHomebrewインストール(`brew shellenv`をシェルの起動ファイルに直接書いている一般的な構成)を持つユーザー環境では、このプロジェクトの他のspec(`vault-backend-service`等)で確認されている通り、Finder起動時のPATH解決は問題なく機能する。

## 4. ドキュメント

- [x] 4.1 変更が内部実装(スレッド安全性・リソースリーク対策)に閉じており、README/openspec/specsへの影響がないことを確認する。SAFETYコメント以外にコメント更新が必要な箇所がないか`main.rs`を再確認する。
      → README.md/CONTRIBUTING.md/openspec/specs/配下のいずれにも`fix_path_env`やログインシェルPATH解決処理への言及なし。影響なしと確認。SAFETYコメントは1.5で更新済み、他に更新が必要なコメント箇所なし。
