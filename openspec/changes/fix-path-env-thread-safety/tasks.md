## 1. 実装: fix_path_env のリファクタリング

- [ ] 1.1 `app/src-tauri/src/main.rs`に、`Command`・タイムアウト値・ポーリング間隔を引数に取り`set_var`を呼ばない関数(例: `run_shell_and_capture_stdout`)を切り出す。内部は`Command::spawn()`で子プロセスを起動し、`try_wait()`をポーリング間隔ごとに呼んでデッドライン内の終了を待つ。
- [ ] 1.2 デッドラインを超過した場合は`child.kill()` → `child.wait()`を呼び、子プロセスを確実にkill・reap(ゾンビ化防止)してから`None`を返す。
- [ ] 1.3 子プロセスが正常終了した場合は、`child.stdout`を`read_to_string`で読み取り(終了済みプロセスの書き込み端は閉じているためブロックしない)、既存の`extract_path_from_marker`でPATHを抽出して返す。
- [ ] 1.4 `fix_path_env()`本体を、「SHELL環境変数を読む→1.1の関数を3秒/50msで呼ぶ→結果があれば`unsafe { std::env::set_var(...) }`する」という薄いラッパーに書き換える。
- [ ] 1.5 `unsafe`ブロックのSAFETYコメントを、「追加スレッドを持たず、子プロセスのkill・reapが完了してから戻るため、この時点で環境変数に触れる他スレッドは存在しない」という、実装によって保証される内容に更新する。

## 2. テスト

- [ ] 2.1 成功パス: `sh -c "echo -n <marker>; printenv PATH"`を実行し、短いtimeout内に完了し期待するPATH文字列が抽出できることを検証するテストを追加する。
- [ ] 2.2 タイムアウト+killパス: `sh -c "sleep 5"`をtimeout=50ms程度・poll_interval=5ms程度まで短縮したパラメータで実行し、(a) `None`が返ること、(b) 壁時計時間で5秒待たずに(timeoutに近い時間で)返ってくることの両方をアサートし、killが実際に効いていることを証明する。
- [ ] 2.3 spawn失敗パス: 存在しないシェルパスを指定してもpanicせず`None`が返ることを検証するテストを追加する。
- [ ] 2.4 既存の`fix_path_env_tests`(`extract_path_from_marker`関連)がリファクタリング後も変更なく通ることを確認する。

## 3. 検証

- [ ] 3.1 `cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings`をすべて通す。
- [ ] 3.2 (手動デバイス確認・ハングケース) `~/.zshrc`等に一時的に`sleep 10`を追加し、Finderから`.app`を起動して次を確認する: (a) アプリがハングせず(3秒程度で)起動すること、(b) タイムアウト後に`ps aux | grep zsh`等でログインシェルの子プロセスが残っていないこと。確認後は`~/.zshrc`の変更を必ず元に戻す。
- [ ] 3.3 (手動デバイス確認・通常ケース) シェルが正常応答する通常環境でもFinder起動時に`bw`コマンドが解決され、アプリが従来通り正常動作すること(リグレッション確認)。

## 4. ドキュメント

- [ ] 4.1 変更が内部実装(スレッド安全性・リソースリーク対策)に閉じており、README/openspec/specsへの影響がないことを確認する。SAFETYコメント以外にコメント更新が必要な箇所がないか`main.rs`を再確認する。
