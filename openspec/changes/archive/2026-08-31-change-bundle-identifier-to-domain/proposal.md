## Why

現在のbundle identifier(`app/src-tauri/tauri.conf.json` の `identifier`)は `com.kuchida1981.bw-quickaccess` で、GitHubユーザー名ベースの逆引きドメイン記法になっている。GitHubアカウント名は将来変わりうる不安定な識別子である一方、所有ドメイン `u-rei.com` はより安定した識別子である。逆引きドメイン記法本来の趣旨(ドメイン所有権に基づく一意性)に沿わせるため、`com.u-rei.bw-quickaccess` に変更する。

現在のアプリ利用者は開発者本人のみであり、macOSのAccessibility権限再許可という既知のコストを許容できる今のタイミングで対応する(issue #118)。

## What Changes

- `app/src-tauri/tauri.conf.json` の `identifier` を `com.kuchida1981.bw-quickaccess` から `com.u-rei.bw-quickaccess` に変更する
- **BREAKING**: 既存インストールでは、macOSのTCC(Accessibility権限、グローバルホットキーに必要)が新しいbundle identifierを別アプリとして扱うため、更新後に一度Accessibility権限の再許可が必要になる(影響は開発者本人の環境のみ)

## Capabilities

### New Capabilities
(なし)

### Modified Capabilities
- `app-bundle-naming`: bundle identifierの値がドメイン所有権に基づいたものであるべきという要件を追加する

## Impact

- 変更ファイルは `app/src-tauri/tauri.conf.json` の1箇所のみ(コードベース調査済み: Keychain/keyring/security-frameworkクレート未使用、identifierに紐づく永続化データなし、コード署名/notarizationパイプライン自体が存在しない、Homebrew Caskはidentifierを参照しない、autostartのLaunchAgentはproductNameに紐づきidentifierとは無関係)
- 唯一の実質的な影響はmacOSのAccessibility権限の再許可(開発者本人の環境のみ)
