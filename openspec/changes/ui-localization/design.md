## Context

現在のUI文言は2箇所にハードコードされている。

1. **Rustネイティブのトレイメニュー**(`app/src-tauri/src/tray.rs`): `status_label()` 等の関数が `&'static str` を直接返す。メニューは起動時に一度だけ `setup_tray()` で構築される。
2. **Webビューのポップアップ**(`app/dist/index.html` / `app/dist/app.js`): ビルドステップなしの素のHTML/JS(`app/`配下に `package.json` は無く、`tauri.conf.json` の `frontendDist` は `../dist` を直接指す)。プレースホルダ・ボタン文言はHTML属性値、フィードバックメッセージ(「ユーザー名をコピーしました」等)はJS内の文字列リテラル。

両者はプロセス空間もビルド方式も異なり、共有できるi18nライブラリは無い。Cargo.tomlには現状 `gettext` 系や `rust-i18n` 等のi18n専用クレートは入っていない。

過去にターミナル版(bash)で同種の対応をした前例(`openspec/changes/archive/2026-08-07-add-i18n-support/`)があるが、`declare -A` 非対応のbash 3.2という制約下での設計であり、GUI版(Rust/TypeScriptなし・vanilla JS)には制約の性質が異なるため機構は作り直す。ただし「明示指定 → OSロケール → フォールバック」という判定順序の考え方は踏襲する。

## Goals / Non-Goals

**Goals:**
- `tray.rs` と `app.js` / `index.html` にハードコードされている全UI文言を、言語別の辞書参照に置き換える
- 言語判定ロジックはRust側に一本化し、JS側は判定結果を問い合わせるだけにする(判定ロジックの二重実装・食い違いを避ける)
- Rust側の辞書は、両言語で項目の過不足があればコンパイルエラーになる形にする(存在しないキーの参照や翻訳漏れを実行時まで気づけない状態を避ける)
- 後続issue(#51 rightキーメニュー、#52 フィールド出し分け、#53 ヘルプ表示、#57 バージョン情報UI等)が新規文言を追加する際、この辞書に項目を足すだけで済む構造にする

**Non-Goals:**
- アプリ内での実行時の言語切り替えUI(トレイメニューからの言語トグル等)は作らない。言語はプロセス起動時に一度だけ決定する(切り替えるにはアプリの再起動が必要という前提でよい)
- 日本語・英語以外の言語追加は対象外(仕組みとして拡張は妨げないが、今回は2言語分のみ用意する)
- 複数形(plural forms)・ICUメッセージフォーマットは対象外(該当する文言が無いため)
- README.md / README.ja.md の言語(英語/日本語)構成自体は既存のまま変更しない。`## Out of scope` の記述更新のみ行う
- #51 / #52 / #53 / #57 等、まだ実装されていないUIの文言追加は本changeのスコープ外(辞書機構と、既存文言の移行のみを行う)

## Decisions

### 1. 言語判定はRust側に一本化し、JS側は結果を問い合わせるだけにする

Tauriの起動処理(`main.rs` / `lib.rs`)で言語を1回だけ判定し、`AppState` 相当の場所に保持する。JS側は新設のTauriコマンド(例: `get_ui_locale() -> "ja" | "en"`)を起動直後に呼び出し、返ってきた値をもとに画面文言を差し替える。

**代替案: JS側も `navigator.language` 等で独自に判定する** — 却下。Rust側とJS側で判定結果が食い違う可能性があり(例: 環境変数オーバーライドをRust側でしか見ていない)、トレイメニューとポップアップで表示言語が一致しない状態を生みうる。判定ロジックを1箇所に持つ方が安全。

### 2. OSロケールの取得方法: シェルの `$LANG` ではなく macOSのシステムロケールAPIを使う

GUIアプリはFinderからの起動やログイン項目からの自動起動が主な起動経路であり、ターミナルの `.zshrc` 等で設定した `$LANG` 環境変数を引き継がない(macOSのGUIアプリはシェルの環境変数を継承しない)。旧bash版が使っていた `LANG`/`LC_ALL` 読み取りをそのまま移植すると、ダブルクリック起動やログイン項目起動では常にフォールバック(英語)に落ちてしまい、実質機能しない。

そのため、macOSのシステム設定(「システム設定 > 一般 > 言語と地域」)を反映するロケールAPI(`sys-locale` クレート等、内部的に `NSLocale`/`CFLocale` を参照する)を使ってOSロケールを取得する。

**代替案: `std::env::var("LANG")` を読む** — 却下。上記の理由でGUI起動経路では機能しないため、開発時(`cargo run` をターミナルから叩く場合)にしか動かない罠になる。

**代替案: `objc2` で `NSLocale` を直接FFI呼び出しする** — 却下(今回は)。`sys-locale` のような薄いクレートで要件を満たせるため、自前でObjective-C FFIを書く必要はない。`objc2` 系の依存は別issue(#56、フォーカス復帰)で必要になった際に改めて検討する。

### 3. 明示的な言語オーバーライドの扱い: 環境変数は「開発時の補助」と位置づける

決定2の理由により、環境変数(例: `BWQA_LANG=en`)によるオーバーライドは、パッケージ済み `.app` をFinderから起動するエンドユーザーには実質使えない(GUIプロセスに環境変数を渡すには `launchctl setenv` 等の追加手順が要る)。一方で `cargo run` によるローカル開発・動作確認時には有用(日本語環境で英語表示を確認する等)。

このchangeでは環境変数オーバーライドは「開発者向けの補助手段」として実装し(判定順序: `BWQA_LANG` 環境変数 → OSロケール → フォールバック `en`)、README等のユーザー向けドキュメントでは案内しない(開発ドキュメント側にのみ記載する)。エンドユーザー向けの言語切り替え手段が今後必要になった場合は、環境変数ではなく設定ファイルやトレイメニューでの切り替え(Non-Goal参照)を別途検討する。

### 4. Rust側の辞書表現: 言語ごとの `Messages` 構造体インスタンス(フラットなフィールド)

```rust
// app/src-tauri/src/i18n.rs (新設)
pub enum Lang { Ja, En }

pub struct Messages {
    pub status_disconnected: &'static str,
    pub status_locked: &'static str,
    pub status_unlocked: &'static str,
    pub hotkey_registered: &'static str,
    // ...
}

const JA: Messages = Messages { status_disconnected: "状態: 未接続", /* ... */ };
const EN: Messages = Messages { status_disconnected: "Status: Disconnected", /* ... */ };

pub fn messages(lang: Lang) -> &'static Messages {
    match lang { Lang::Ja => &JA, Lang::En => &EN }
}
```

`tray.rs` 等は `let m = i18n::messages(lang);` のあと `m.status_disconnected` のようにフィールドアクセスする。

**代替案: `HashMap<&str, &str>` によるキー文字列ルックアップ** — 却下。キー名のtypoや翻訳漏れが実行時(かつ該当パスを通ったときのみ)にしか発覚しない。構造体フィールドにすればコンパイラが「`JA` にはあるが `EN` に無いフィールド」を検出でき、翻訳漏れを防げる。

**代替案: `rust-i18n` 等の専用クレート導入** — 却下。マクロベースでビルド時に外部YAML/JSONを読み込む方式が主流だが、対象文言が数十件規模で今後も緩やかにしか増えない見込みであり、新規マクロ依存を増やすほどの規模ではない。

### 5. JS側の辞書表現とDOM適用方法: `data-i18n` 属性 + 素のJSオブジェクト辞書

ビルドステップが無い制約上、テンプレートエンジンやJSXは使わない。

- `app/dist/i18n.js`(新設、`index.html` から `app.js` より先に `<script>` で読み込む)に `const MESSAGES = { ja: {...}, en: {...} }` を定義する
- 静的なHTML要素(プレースホルダ・ボタン文言・見出し等)には `data-i18n="unlockButton"` / `data-i18n-placeholder="searchPlaceholder"` のような属性を付け、起動時に1回だけ全要素を走査して `textContent` / `placeholder` を差し替える
- `app.js` 内で動的に組み立てているフィードバック文言(`runAction` に渡している `"ユーザー名をコピーしました"` 等の文字列リテラル)は、該当箇所を `t("feedback.copyUsername")` のような参照に置き換える

**代替案: HTML自体を `index.ja.html` / `index.en.html` の2ファイルに分岐する** — 却下。Rust側からWebViewに読み込むHTMLファイル自体を出し分ける仕組みが必要になり、`tauri.conf.json` の `frontendDist` 構成が複雑化する。動的な言語判定(決定1)とも相性が悪い。

## Risks / Trade-offs

- [Risk] `sys-locale` 等の新規クレート追加により、依存ツリー・ビルド時間がわずかに増える → [Mitigation] `sys-locale` は軽量・依存少なめの実績あるクレートを選定する。導入前に `cargo tree` で影響を確認する
- [Risk] 環境変数オーバーライドが「開発時のみ有効」という制約を、実装時にドキュメント化し忘れると、将来のissueで「なぜユーザー向けに案内しないのか」が分からなくなる → [Mitigation] `i18n.rs` のコメントとREADMEの開発者向けセクション(あれば)に明記する
- [Risk] `data-i18n` 属性の付与漏れにより、一部の要素だけ翻訳されない「バイリンガル画面」になる → [Mitigation] tasks.md で `index.html` の全テキストノード・`app.js` の全文字列リテラルを洗い出すタスクを明示し、レビュー時に日英両方で目視確認する
- [Trade-off] Rust側は構造体、JS側はオブジェクトと、2つの異なる仕組みを保守することになる(共通化は困難: 言語もランタイムも別)。今回は許容し、両者のキー命名規則だけ揃える(例: `status_disconnected` ↔ `status.disconnected`)ことで対応関係を分かりやすくする

## Migration Plan

- 既存の日本語ハードコード文字列を、`JA` 側の辞書値としてそのまま移し、その後 `EN` 側の翻訳を追加する(挙動が変わらないことをまず担保してから翻訳を追加する2段階で進める)
- 破壊的変更なし。既存ユーザー(日本語ロケール環境)は導入後も従来通り日本語表示になる
- ロールバック: 辞書参照への置き換えは対象ファイル(`tray.rs` / `app.js` / `index.html` / 新設 `i18n.rs` / `i18n.js`)に閉じており、問題があればこのchangeの変更のみをrevertすれば旧状態に戻せる

## Open Questions

- 環境変数オーバーライド(`BWQA_LANG`)は本当に用意する価値があるか、それとも今回はOSロケール判定のみ実装し、オーバーライドは需要が出てから追加する形でも良いか(決定3参照)
- `sys-locale` クレートの採用可否(ライセンス・メンテナンス状況)は実装着手時に改めて確認する
