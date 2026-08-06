# DOCS_RS=1 でのビルドスクリプト実行後に通常ビルドが失敗する

- Created: 2026-08-02
- Completed: (未完了)
- Branch: feature/fix-docs-rs-build-script
- Polished: 2026-08-06
- Reporter: @voluntas

## 目的

docs.rs 向けのダミーバインディング生成が通常ビルドを壊さないようにする。

## 現状

`build.rs` は `DOCS_RS` 環境変数が設定されていると、4 つの型定義だけのダミーバインディング (`bindings.rs`) を `OUT_DIR` に書き込む。

`cargo::rerun-if-env-changed` に `DOCS_RS` が含まれないため、一度 `DOCS_RS=1` でビルドスクリプトが実行されると、その後の通常の `cargo build` でビルドスクリプトが再実行されずダミーが使われ続け、`EbErrorType_EB_ErrorNone` などのシンボルが無いことによるコンパイルエラーで失敗する。fingerprint はターゲットディレクトリごとに記録されるため、この再現は同一ターゲットディレクトリでのみ起こる。回復には `touch build.rs` か `cargo clean` が必要。

再現手順:

1. `cargo clean -p shiguredo_svt_av1` でビルドスクリプトの実行結果と fingerprint を破棄する (既存の fingerprint が残っているとビルドスクリプトが再実行されず、ダミーが書き込まれないため)
2. `DOCS_RS=1 cargo build` を実行する (ビルドスクリプトがダミーを書き込み、その後のクレートコンパイルがシンボル不足で失敗する)
3. 続けて通常の `cargo build` を実行する (ビルドスクリプトが再実行されず、ダミーが使われ続けて失敗する)

実機で再現済み。

## 設計方針

`DOCS_RS` を `cargo::rerun-if-env-changed` に追加し、環境変数の切り替えでビルドスクリプトが再実行されるようにする。

ダミーバインディングは `DOCS_RS` 環境変数が設定されている場合のみ生成し、コンパイル不能にするダミーのまま維持する。cargo はビルドスクリプトに `cargo doc` と `cargo build` を区別する環境変数を提供しないため、ビルドスクリプトから `cargo doc` を検出することはできない。docs.rs のビルドは `cargo rustdoc --lib` のみを実行する (2026-08 時点の rust-lang/docs.rs ソースで確認。テストやビルドは実行しない) ため、この分岐によりダミーは実質的に docs.rs 向けドキュメント生成でのみ使用される。また、rustdoc は関数本体を型チェックしないため、コンパイル不能にするダミーを含むクレートでも `cargo doc` は成功する。コンパイル不能のまま維持するのは、誤って通常ビルドでダミーが使われた場合にコンパイルエラーとして検出できる保護になるため。

## 完了条件

- 再現手順で修正前の失敗を確認したうえで、修正後の状態で以下が成功すること (この条件が修正の主検証):
  1. `cargo clean -p shiguredo_svt_av1` を実行してビルドスクリプトの実行結果と fingerprint を破棄する (既存の fingerprint が残っているとビルドスクリプトが再実行されずダミーが書き込まれないため、修正前でも成功してしまう)
  2. 同一ターゲットディレクトリで `DOCS_RS=1 cargo doc --no-deps` を実行する (修正前はここでダミーが書き込まれ、続く通常の `cargo build` が失敗する)
  3. 続けて通常の `cargo build` が成功すること
  - 検証は prebuilt バイナリが存在する環境で行うこと。存在しない環境 (リリース前の新 OS 等) では prebuilt ダウンロードの失敗で別原因のエラーになり、修正の成否を誤判定するため
- docs.rs 向けのドキュメント生成が引き続き成功すること (`.github/workflows/ci.yml` の docs-rs job が通ること。この条件は回帰確認であり、修正前も満たされる。`DOCS_RS=1 cargo doc --no-deps` のローカル確認は完了条件 1 の手順に含まれる)
- `CHANGES.md` の develop セクションに [FIX] として追記すること

## 解決方法

- `build.rs` の既存の `cargo::rerun-if-env-changed=CARGO_FEATURE_SOURCE_BUILD` の直後に、`cargo::rerun-if-env-changed=DOCS_RS` を追加する
- 修正後は `DOCS_RS` の切り替えでビルドスクリプトが再実行され、prebuilt の再ダウンロードが発生するが、これは正常動作であり問題ない
