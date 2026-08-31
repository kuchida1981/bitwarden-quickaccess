## ADDED Requirements

### Requirement: 所有ドメインに基づくバンドル識別子
バンドル識別子(`tauri.conf.json` の `identifier`)は、開発者が所有するドメインに基づく逆引きドメイン記法でなければならない(SHALL)。GitHubアカウント名など、開発者本人の意思によらず変わりうる識別子をベースにしてはならない(SHALL NOT)。

#### Scenario: バンドル識別子が所有ドメインに基づいている
- **WHEN** `tauri.conf.json` の `identifier` を確認する
- **THEN** 値は開発者が所有するドメイン `u-rei.com` に基づく `com.u-rei.bw-quickaccess` である
