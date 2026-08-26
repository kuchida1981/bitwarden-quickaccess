# homebrew-distribution

## Purpose

既存のインストール手段(GitHub Releasesからの手動ダウンロード、セルフビルド)に加えて、Homebrew tap経由での `brew install --cask` によるインストールを提供し、より手軽な導入手段を確保する。tap/Cask自体は別リポジトリ(`kuchida1981/homebrew-bitwarden-quickaccess`)の成果物であるため、本リポジトリ内で検証可能な要件はREADMEのドキュメント記載に限定される。

## Requirements

### Requirement: Homebrew経由のインストール手順の提供
README.md および README.ja.md は、Homebrew tap経由でのインストール手順(`brew tap` および `brew install --cask` のコマンド)を、既存のインストール手段(GitHub Releasesからの手動ダウンロード、セルフビルド)と並べて記載しなければならない(SHALL)。

#### Scenario: READMEにHomebrewの手順が記載されている
- **WHEN** README.md の `## Install` セクションを確認する
- **THEN** `brew tap kuchida1981/bitwarden-quickaccess` と `brew install --cask bw-quickaccess` のコマンドが手順として記載されている

### Requirement: Gatekeeper制約の明示
README.md および README.ja.md は、Homebrew経由でインストールした場合であっても、本アプリが無署名・非notarizedであることに起因するGatekeeperの警告が発生しうることを明記しなければならない(SHALL)。

#### Scenario: Homebrewインストール後もGatekeeper対応が必要なことが分かる
- **WHEN** README.md のHomebrewインストール手順を確認する
- **THEN** Gatekeeper警告への対処方法(右クリックで開く、または `--no-quarantine` オプション)が記載されている
