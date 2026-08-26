## Context

`app/src-tauri/src/popup.rs` は以下の2関数からなる。

- `create_popup_window(app: &AppHandle)`: アプリ起動時に一度だけ呼ばれ、ポップアップウィンドウを生成する。現状、`app.primary_monitor()` を使って画面上部中央の座標を一度だけ計算し、`WebviewWindowBuilder::position()` で固定する。以降ウィンドウの位置は変わらない。
- `toggle_popup(app: &AppHandle)`: ホットキー押下のたびに呼ばれ、`window.show()` / `window.hide()` をトグルする。位置の再計算やフォーカス関連の処理は一切ない。

このchangeで対応する2つの要求(issue #55, #56)は、いずれも「`toggle_popup` の show/hide のタイミングでOS側のウィンドウ・ディスプレイ情報を取得・反映する」という共通の実装ポイントを持つ。

事前調査により、以下がプロジェクトの実際の依存バージョン(`tauri 2.11.5` / `objc2-app-kit 0.3.2`、いずれも `Cargo.lock` で解決済み)で確認できている:

- `AppHandle::cursor_position() -> Result<PhysicalPosition<f64>>`(`tauri` 標準API、`src/app.rs`)
- `AppHandle::monitor_from_point(x: f64, y: f64) -> Result<Option<Monitor>>`(同上)
- `objc2_app_kit::NSWorkspace::sharedWorkspace() -> Retained<NSWorkspace>`
- `NSWorkspace::frontmostApplication(&self) -> Option<Retained<NSRunningApplication>>`
- `NSRunningApplication::processIdentifier(&self) -> libc::pid_t`
- `NSRunningApplication::runningApplicationWithProcessIdentifier(pid: libc::pid_t) -> Option<Retained<Self>>`(クラスメソッド)
- `NSRunningApplication::activateWithOptions(&self, options: NSApplicationActivationOptions) -> bool`

`objc2-app-kit` は `tauri`/`tauri-runtime-wry` 等を通じて既に `Cargo.lock` に間接依存として含まれており(`tray-icon`/ウィンドウ管理のmacOSバックエンドが利用)、`NSWorkspace`・`NSRunningApplication` はクレートの `default` feature に含まれる。そのため直接依存として追加しても新規の推移的依存ツリーの拡大は実質的に発生しない。

## Goals / Non-Goals

**Goals:**
- ホットキー押下時点でカーソルがあるディスプレイにポップアップを表示する(#55)
- ポップアップを表示する直前にフォアグラウンドだった他アプリケーションを、ポップアップが非表示になったタイミングで再度アクティブ化する(#56)
- 新規の重い外部依存(自前のObjective-C FFI実装、追加の推移的依存の大幅増加)を避ける

**Non-Goals:**
- 「アクティブウィンドウがあるディスプレイ」の厳密な判定(前面ウィンドウのフレーム座標を取得してそのディスプレイを特定する等)は行わない。カーソル位置を代理指標として使う簡易な近似で十分とする(決定1参照)
- 表示先ディスプレイの手動指定・設定UIは作らない
- フォーカス復帰の対象は直前1アプリケーションのみとし、複数階層の履歴(スタック)は追跡しない
- ポップアップ表示中に他アプリへの切り替えが発生した場合の追跡・キャンセル処理(例: ポップアップを開いたまま手動で別アプリに切り替えた場合の挙動)は変更しない。あくまで「ポップアップを開く直前にフォアグラウンドだったアプリ」を1回記録し、閉じるときに1回復元するだけの単純な仕組みとする

## Decisions

### 1. 「アクティブなディスプレイ」はホットキー押下時点のカーソル位置で代理する

1Password Quick Accessの「アクティブウィンドウがあるディスプレイに表示する」という挙動を厳密に再現するには、macOSのアクセシビリティAPI等でフォーカス中のウィンドウのフレーム座標を取得する必要があり、実装コストと権限要求(アクセシビリティ権限)が増える。マウスカーソルは通常、ユーザーが直前まで操作していたディスプレイ上にあることが多いため、`AppHandle::cursor_position()` → `AppHandle::monitor_from_point(x, y)` で求めたディスプレイを「アクティブなディスプレイ」の代理指標として採用する。

**代替案: アクセシビリティAPI(`AXUIElement`)でフォーカスウィンドウのフレームを取得** — 却下。実装コストが高く、初回に無人環境設定 > プライバシーとセキュリティ > アクセシビリティでの権限許可が必要になり、ユーザー体験上のハードルが上がる。カーソル位置ベースの近似で当面の要求を満たせると判断。

`monitor_from_point` が `None` を返す場合(取得失敗、マルチディスプレイ環境の座標系の端等)は、既存の `primary_monitor()` にフォールバックする(現状の挙動を維持)。

### 2. 位置計算のタイミングを `create_popup_window` から `toggle_popup` の表示直前に移す

現状は起動時に一度だけ位置を計算しウィンドウ生成時に固定しているが、毎回のホットキー押下時点のカーソル位置に追従させるには、`window.show()` を呼ぶ直前に毎回 `window.set_position(...)` で位置を再計算・再設定する必要がある。ウィンドウ自体の生成(`create_popup_window`)は従来通り起動時に一度だけ行う(生成コスト自体は変えない)。

### 3. フォーカス復帰は「PID(プロセスID)を記録し、非表示時に再取得して呼び出す」方式にする

`NSRunningApplication`(`Retained<NSRunningApplication>`)を直接どこかに保持し続ける方式は避け、`processIdentifier()`(`libc::pid_t`、ただの`i32`)のみをアプリ全体の状態(Tauriの `.manage()`)に記録する。非表示にする直前に、記録したPIDから `NSRunningApplication::runningApplicationWithProcessIdentifier(pid)` で改めて `NSRunningApplication` インスタンスを取得し、それに対して `activateWithOptions` を呼ぶ。

**代替案: `Retained<NSRunningApplication>` をそのままアプリ状態として保持する** — 却下。Tauriの `.manage()` は値が `Send + Sync + 'static` であることを要求するが、Cocoaのオブジェクト(`Retained<T>`)は一般にスレッドセーフ性が保証されておらず `Send`/`Sync` を実装しない。PIDという単純な整数値だけを保持すれば、この制約を気にする必要がなく、かつ「記録した時点のアプリが既に終了していた」場合も `runningApplicationWithProcessIdentifier` が `None` を返すだけなので自然に無視できる。

新しい管理状態として `struct PreviousFrontmostApp(Mutex<Option<libc::pid_t>>)` を定義し、`main.rs` で `.manage(PreviousFrontmostApp(Mutex::new(None)))` する。

- `toggle_popup` の表示(`show()`)分岐: `window.show()` の**前**に `NSWorkspace::sharedWorkspace().frontmostApplication()` を呼び、そのPIDを `PreviousFrontmostApp` に保存する
- `toggle_popup` の非表示(`hide()`)分岐、および `commands::hide_popup` コマンド(コピー操作後にフロントエンドから呼ばれる、`app/dist/app.js` の `runAction` 参照)から呼ばれる非表示処理: `window.hide()` の**後**に `PreviousFrontmostApp` からPIDを取り出し(`take()` して`None`に戻す)、`runningApplicationWithProcessIdentifier` で取得できれば `activateWithOptions` を呼ぶ

### 4. macOS専用コードとして扱う(`#[cfg(target_os = "macos")]` によるガードは不要)

このプロジェクトはmacOS専用アプリであり(README「Requirements: macOS」、CIも`macos-latest`のみ)、`objc2-app-kit` はmacOS/iOS専用クレートだが、他OSでのビルドを現状サポートしていないため追加のcfgガードは不要と判断する。将来Linux対応する際は、この部分をまとめて `#[cfg(target_os = "macos")]` で分離することを検討する(このchangeのスコープ外)。

## Risks / Trade-offs

- [Risk] カーソル位置とアクティブウィンドウの位置が一致しないケース(例: 外部ディスプレイのウィンドウを操作しながらマウスカーソルだけプライマリディスプレイに置いている)では、意図と異なるディスプレイにポップアップが表示される → [Mitigation] Non-Goalとして明記し、実用上ほとんどのケース(直前にクリック・入力していたディスプレイにカーソルが残る)をカバーできれば十分とする
- [Risk] `activateWithOptions` の具体的な挙動(フォーカスが確実に奪えるか、`NSApplicationActivationOptions` にどのオプションを渡すべきか)はmacOSのバージョンやセキュリティ設定に依存する可能性があり、ソースコード調査だけでは断定できない → [Mitigation] 実装時にまず `NSApplicationActivationOptions::empty()` で試し、実機確認(タスク側で担当)で期待通りフォーカスが戻らない場合に `NSApplicationActivateIgnoringOtherApps` 相当のオプションを試す、という2段階で進める
- [Risk] AppKitのAPI呼び出しはメインスレッドから行う必要があるという一般的な制約があるが、`toggle_popup` はホットキーハンドラ(`main.rs` の `with_handler` コールバック)から同期的に呼ばれており、Tauriのイベントループはメインスレッドで実行されるため、通常は問題にならない見込み → [Mitigation] 実装時に別スレッド・非同期タスクから呼び出していないことを確認する
- [Trade-off] PIDベースの記録は、記録してから復元するまでの間に同じPIDが別プロセスに再利用される可能性は理論上あるが、ポップアップの表示から非表示までは数秒程度の短時間であり、実用上のリスクは無視できる
