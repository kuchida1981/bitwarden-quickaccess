# open-in-browser-action

## Purpose

検索結果一覧でフォーカスされている行に対し、`Enter` キー押下でそのアイテムのURL(`login.uris[0].uri`)をデフォルトブラウザで開く機能を提供する。1Password Quick Access相当の体験の一部として、URLを持つログインアイテムへの素早いアクセスを可能にする。

## Requirements

### Requirement: Enterキーによるブラウザ起動
一覧の行にフォーカスがある状態で `Enter` が押下された場合、そのアイテムのURL(`login.uris` の先頭要素)がデフォルトブラウザで開かれなければならない(SHALL)。

#### Scenario: フォーカス行のURLをブラウザで開く
- **WHEN** URLが設定されたアイテムの行にフォーカスがある状態で `Enter` が押下される
- **THEN** そのアイテムの先頭のURLがデフォルトブラウザで開かれる

### Requirement: URL未設定時のフィードバック
フォーカス行のアイテムにURLが設定されていない場合、`Enter` 押下時にブラウザは起動せず、その旨のフィードバックが表示されなければならない(SHALL)。

#### Scenario: URLがないアイテムでEnterを押す
- **WHEN** URLが設定されていないアイテムの行にフォーカスがある状態で `Enter` が押下される
- **THEN** ブラウザは起動せず、URLが存在しない旨のフィードバックが表示される
