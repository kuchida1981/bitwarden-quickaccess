## Context

See proposal.md - Why. `/opsx:explore` での調査(本change作成前の対話)で、`identifier` への参照・依存箇所を洗い出した:

- `identifier` への参照は `app/src-tauri/tauri.conf.json` の1箇所のみ。Keychain/keyring/security-frameworkクレートは未使用で、`identifier` に紐づく永続化データ(app_data_dir等)も存在しない。
- コード署名/notarizationパイプラインは `release.yml` に存在しない(`APPLE_*`系シークレット未設定)ため、この観点での影響はない。
- Homebrew Cask(`kuchida1981/homebrew-bitwarden-quickaccess`)は `productName` とcask token(`bw-quickaccess`)のみを参照し、`identifier` は参照しない。
- autostart(`tauri-plugin-autostart`)のLaunchAgent plistは `productName` に紐づく実装(`auto-launch` crateの挙動、[[app-bundle-naming spec]] archiveの `2026-08-27-rename-product-name/design.md` で確認済み)であり、`identifier` とは無関係。今回 `productName` は変更しないため影響なし。
- 唯一の実質的な影響は、macOSのTCC(Transparency, Consent, Control)。グローバルホットキー(`tauri_plugin_global_shortcut`、Shift+Cmd+Space)に必要なAccessibility権限は「コード署名 + bundle identifier」の組で記憶されるため、`identifier` 変更後は既存インストールで別アプリ扱いとなり、再許可が必要になる。現在の利用者は開発者本人のみであるため、このコストは許容する。

## Goals / Non-Goals

**Goals:**
- `tauri.conf.json` の `identifier` を `com.u-rei.bw-quickaccess` に変更する
- 変更に伴うAccessibility権限の再許可が必要になる旨を明文化し、想定内のコストとして扱う

**Non-Goals:**
- `productName` / `Cargo.toml` のパッケージ名・バイナリ名の変更([[app-bundle-naming]] の既存要件により対象外)
- 旧Accessibility権限エントリの検出・案内コードの実装(現状のエラーメッセージで十分と判断)
- Homebrew Cask・release.yml への変更(identifierを参照していないため不要)

## Decisions

### 1. `identifier` を `com.u-rei.bw-quickaccess` にする
所有ドメイン `u-rei.com` を反転した記法。`bw-quickaccess` の部分は既存のCask token・バイナリ名(`bw-quickaccess-gui`)との一貫性を保つため変更しない。

### 2. コード内でのAccessibility権限の事前案内・移行コードは追加しない
利用者が開発者本人のみであるため、更新後にホットキーが効かなくなった場合は既存のエラーメッセージ(`hotkey_registration_failed`、Accessibility設定への案内を含む)で十分に対応できる。専用の移行コード・通知を追加するコストに見合わない。

## Risks / Trade-offs

- [Risk] 更新後、既存インストールでグローバルホットキーが動作しなくなる(Accessibility権限が新しい `identifier` に対して未許可のため) → [Mitigation] 利用者は開発者本人のみであり、System Settings > Privacy & Security > Accessibility で再許可すれば解消する。既存のエラーメッセージがこの手順を案内している。
- [Risk] 将来利用者が増えた状態で同様の `identifier` 変更が必要になった場合、再許可の影響範囲が拡大する → [Mitigation] 今回追加する新要件([[app-bundle-naming]] の「所有ドメインに基づくバンドル識別子」)により、`identifier` は安定した所有ドメインに基づく値となるため、今後同様の変更が発生する可能性は低い。

## Migration Plan

1. `app/src-tauri/tauri.conf.json` の `identifier` を `com.kuchida1981.bw-quickaccess` から `com.u-rei.bw-quickaccess` に変更する
2. `cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings` を実行し、ビルドに影響がないことを確認する
3. ローカルで `cargo tauri build`(または既存のビルド手順)を実行し、生成される `.app` バンドルの `Info.plist` の `CFBundleIdentifier` が新しい値になっていることを確認する
4. 開発者本人の環境で新しいビルドに更新し、グローバルホットキーが動作するか確認する。動作しない場合はSystem Settings > Privacy & Security > Accessibilityで新しいアプリのエントリを許可する

ロールバックは `identifier` を元の値に戻すのみで完結する。
