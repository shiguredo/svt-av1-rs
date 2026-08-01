# DOCS_RS=1 でのビルドスクリプト実行後に通常ビルドが失敗する

- Created: 2026-08-02
- Completed: (未完了)
- Branch: feature/fix-docs-rs-build-script
- Polished: (未実施)
- Reporter: @voluntas

## 目的

docs.rs 向けのダミーバインディング生成が通常ビルドを壊さないようにする。

## 現状

`build.rs` は `DOCS_RS` 環境変数が設定されていると、4 つの型定義だけのダミーバインディング (`bindings.rs`) を `OUT_DIR` に書き込む。

`cargo::rerun-if-env-changed` に `DOCS_RS` が含まれないため、一度 `DOCS_RS=1` でビルドスクリプトが実行されると、その後の通常の `cargo build` でビルドスクリプトが再実行されずダミーが使われ続け、`EbErrorType_EB_ErrorNone` 等の定数が無いことによるコンパイルエラー (実機で 137 エラー) で失敗する。回復には `touch build.rs` か `cargo clean` が必要。

実機で再現済み。

## 設計方針

`DOCS_RS` を `cargo::rerun-if-env-changed` に追加し、環境変数の切り替えでビルドスクリプトが再実行されるようにする。あわせて、ダミーバインディングを `cargo doc` でのみ使用する。

## 完了条件

- `DOCS_RS=1 cargo doc --no-deps` を実行した後に、通常の `cargo build` が成功すること
- docs.rs 向けのドキュメント生成が引き続き成功すること

## 解決方法

- `build.rs` に `cargo::rerun-if-env-changed=DOCS_RS` を追加する
- ダミーバインディングがコンパイル不能であること (rustdoc が関数本体を型チェックしないため `cargo doc` のみ成功する) も確認し、可能ならダミーに必要な定数・関数を追加する
