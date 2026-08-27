## Context

Issue #68 のコメントで、Finder上のアプリ名 `bw-quickaccess` が無機質だという指摘があった。PR #107（`rename-app-display-name`）ではアプリ内表示テキスト（ウィンドウタイトル・アンロック画面見出し）のみを変更し、`productName` は Homebrew 配布・自動起動への影響が未整理だったため意図的に据え置いた。

`/opsx:explore` での調査（このchange作成前の対話）で、`productName` を変更した場合の具体的な影響を確認した:

- `tauri_plugin_autostart` はデフォルトで `app.package_info().name`（= `productName`）を LaunchAgent の識別子として使い、plist ファイル名は `~/Library/LaunchAgents/{productName}.plist` になる（`auto-launch` crate のソースで確認済み）。`productName` を変えると、既存インストールの旧 plist は孤児化し、新しい設定は新ファイル名を見るため自動起動の表示がリセットされる。
- リリースアセット名（`bw-quickaccess_aarch64.app.tar.gz`）も `productName` に由来する。スペースを含む `productName` にすると、生成される `.app` / tar.gz 名にもスペースが入る。
- Homebrew tap リポジトリ（`kuchida1981/homebrew-bitwarden-quickaccess`、別リポジトリ）の Cask 定義には `app "bw-quickaccess.app"` および `caveats` 文中のファイル名参照があるが、`.github/workflows/release.yml` の `brew bump-cask-pr` ステップは `version`/`sha256` のみを自動更新し、これらのファイル名参照までは追従しない。
- 現在のアプリ利用者は開発者本人のみであり、検証中の段階である。

## Goals / Non-Goals

**Goals:**
- Finder上の表示名（`productName`）を "Bitwarden Quick Access" に変更する
- 既存の `identifier` と `Cargo.toml` のパッケージ名・バイナリ名は変更しない
- README.md / README.ja.md のファイル名記載を新しい名称に追従させる
- 今回発生する Homebrew Cask の非自動更新箇所を、CI変更なしで安全に運用できる手順として残す

**Non-Goals:**
- `release.yml` への Cask `app`/`url`/`caveats` 自動更新ロジックの追加
- 旧 LaunchAgent plist を検出・削除する移行コードの実装
- バンドル識別子（`identifier`）やコード署名・notarizationの変更
- Homebrew tap リポジトリ自体のコード変更（本changeのスコープ外、手動作業のみ）

## Decisions

### 1. `productName` は "Bitwarden Quick Access"（スペースあり）とする
Finder上の表示名としての自然さを優先する。スペースなし（`BitwardenQuickAccess`）やハイフン区切りも検討したが、表示の親しみやすさというIssue #108の主目的に対して劣るため採用しない。スペースを含むことで発生するCask `url` 行のエンコード対応は、後述の通り手動運用で吸収できる小さなコストと判断した。

### 2. `identifier` と `Cargo.toml` のパッケージ名・バイナリ名は変更しない
`identifier` を変えると、macOSがアクセシビリティ権限（グローバルホットキーに必要）を別アプリとして扱う可能性があり、既存ユーザーへの影響が大きい。`Cargo.toml` のパッケージ名/バイナリ名もビルド成果物パスや内部参照に波及するため、今回のスコープからは明示的に除外する。

### 3. Homebrew Cask の `app`/`url`/`caveats` 更新は、CI自動化せず手動の一回限りの対応とする
`brew bump-cask-pr` は `version`/`sha256` のみを更新する設計であり、ファイル名参照までを安全に書き換える自動化（スペースのパーセントエンコード含む）を `release.yml` に追加するのは、今回一度きりのイベントに対して見合わないメンテナンスコストになる。現在の利用者が開発者本人のみであることも踏まえ、次回リリース時に手動でtapリポジトリの `Casks/bw-quickaccess.rb` を編集する運用とし、その手順を `CONTRIBUTING.md` に注記する。

### 4. 旧 LaunchAgent plist の孤児化に対する移行コードは実装しない
現在の利用者が開発者本人のみで検証中であるため、影響は「アップデート後に一度、自動起動をオン/オフし直す（または不要な plist を手動削除する）」程度に留まる。設計上の既知の制約として記録するに留め、コード変更は行わない。将来的に利用者が増えた場合は別途対応を検討する（Open Questions参照）。

### 5. README.md / README.ja.md のファイル名記載は本changeで更新する
`bw-quickaccess.app` / `bw-quickaccess_aarch64.app.tar.gz` という記載はユーザー向けの実際の手順であり、`productName` 変更後は不正確になる。エンドユーザー向けドキュメントの正確性に直結するため、これは本changeのスコープに含める。

## Risks / Trade-offs

- [Risk] Homebrew Cask の `url` 行にスペースを含む生のファイル名を埋め込むと、`brew fetch`/`brew install` がURLを正しく解決できない → [Mitigation] 手動編集時にスペースを `%20` にパーセントエンコードした文字列を `url` 行に使う。`CONTRIBUTING.md` にこの注意点を明記する。
- [Risk] 既存インストール（開発者本人の環境）で、アップデート後に自動起動の表示が一時的にリセットされ、孤児化した旧 plist ファイルが残る → [Mitigation] コード対応はせず、アップデート後に手動でトグルし直す、または `rm ~/Library/LaunchAgents/bw-quickaccess.plist` を実行する。影響は開発者本人のみに限定される。
- [Risk] 手動でのtap更新を忘れると、次回リリースの `brew install --cask` が壊れたURLを指したままになる → [Mitigation] `CONTRIBUTING.md` のリリース手順チェックリストに、このリリース限定の手動編集ステップと `brew style`/`brew audit`/`brew reinstall --cask` による確認を追記する（既存のリリース手順と同じ確認フローに乗せる）。
- [Risk] 将来的に利用者が増えた状態で同様の `productName` 変更を再度行うと、今回と同じ手動対応・自動起動リセットが再発する → [Mitigation] 今回はスコープ外とするが、Open Questionsに残し、利用者が増えた段階で自動化・移行コードへの投資を再検討する。

## Migration Plan

1. `app/src-tauri/tauri.conf.json` の `productName` を `"Bitwarden Quick Access"` に変更する
2. README.md / README.ja.md 内のファイル名記載（`.app` / `.app.tar.gz`）を新しい名称ベースに更新する
3. `CONTRIBUTING.md` のリリース手順に、Homebrew tap の `app`/`url`/`caveats` を手動更新する旨の注記（スペースのパーセントエンコード方法を含む）を追加する
4. ローカルで `cargo tauri build`（または既存のビルド手順）を実行し、生成される `.app` バンドル名・アセット名を確認する
5. `cargo test` / `cargo clippy --all-targets -- -D warnings` を実行する
6. 次回リリース時、手動でtapリポジトリの `Casks/bw-quickaccess.rb` を編集し、`brew style`/`brew audit`/`brew reinstall --cask` で確認する
7. 開発者本人の環境で、アップデート後に自動起動設定を確認し、必要であれば手動でトグルし直す

ロールバックは `productName` とREADME記載を元の値に戻すのみで完結する（`identifier` を変更していないため、権限まわりの複雑なロールバックは発生しない）。

## Open Questions

- 将来的に利用者が増えた場合、Homebrew Cask のファイル名更新自動化（`release.yml` でのスペースエンコード対応込みのsedステップ追加など）に投資すべきか?
- 同様に、旧 LaunchAgent plist を検出して削除する移行コードをアプリ起動時に追加すべきか? (現時点では見送り)
