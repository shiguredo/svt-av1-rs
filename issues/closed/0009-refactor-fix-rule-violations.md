# プロジェクト規約違反を是正する

- Created: 2026-08-02
- Completed: 2026-08-06
- Branch: feature/refactor-fix-rule-violations
- Polished: 2026-08-06
- Reporter: @voluntas

## 目的

AGENTS.md と shiguredo-rust 規約への違反を是正する。本 issue は下記 4 件のみを対象とし、他の規約違反 (例: `src/lib.rs` の `.unwrap()` の使用) は対象外とする。

## 現状

- テストの assert / expect メッセージに英語が残っている (AGENTS.md「テストのログメッセージは全て日本語にすること」違反): `src/lib.rs` のテストと `tests/test_psnr_aom.rs` / `tests/test_psnr_dav1d.rs`。テストコードに `println!` / `eprintln!` 等のログ出力は存在せず、テスト失敗時に表示される expect / assert メッセージを「テストのログメッセージ」と解釈する (既存の日本語 expect「エンコーダーの生成に失敗」等が先例)
- Cargo.toml の依存に用途コメントがない (shiguredo-rust 規約「依存ライブラリには用途をコメントで明記すること」違反)
- `.github/workflows/ci.yml` の clippy が `--all-targets` なし (prek.toml との不整合。CI の clippy でテストコードが検査されない)。`Makefile` の clippy ターゲットも同様
- prek.toml の cargo-test に `pre-push` ステージの指定がない (規約「cargo test は pre-push ステージだけで実行すること」違反)。tombi の lint / format フックもない

## 設計方針

- ライブラリの動作は変えず、テストメッセージ・コメント・CI / prek の設定のみを修正する
- CI とローカル (prek) の検査設定を一致させる
- prek の設定は shiguredo-rust スキルの参考設定 prek.toml に従う

## 完了条件

- テストの英語 assert / expect メッセージが日本語になっていること (expect と assert のメッセージ引数に限定した grep で英語メッセージ 0 件を確認。技術用語 (PSNR / dB 等) は許容)。ただし assert の検証対象文字列 (ライブラリのエラーメッセージ。例: `assert!(err.to_string().contains("Y plane size mismatch"))`) は対象外 (ライブラリのエラーメッセージは AGENTS.md により英語)
- `Cargo.toml` の各依存に用途コメントがあること (grep でコメントなし依存 0 件を確認)
- `ci.yml` と `Makefile` の clippy に `--all-targets` が含まれ、CI が通ること
- `prek.toml` の cargo-test に `stages = ["pre-push"]` があり、tombi フックがあり、`prek validate-config` が通ること。`prek run --all-files` は pre-commit ステージのフックのみを実行するため、cargo-test の実行は CI で担保する
- `CHANGES.md` の develop セクションの misc に [UPDATE] として追記すること

## 解決方法

- テストの英語 assert / expect メッセージを日本語に変更した (assert の検証対象文字列 (ライブラリのエラーメッセージ) は対象外として英語のまま)
  - `src/lib.rs` の `#[cfg(test)]` モジュール: "failed to create" / "failed to encode" / "failed to finish" を日本語に統一した
  - `tests/test_psnr_aom.rs` / `tests/test_psnr_dav1d.rs`: エンコーダー・デコーダー生成、エンコード・デコード、finish、PSNR 検証の assert メッセージを日本語に統一した (技術用語 (PSNR / dB) は許容)
- `Cargo.toml` の各依存 (log / shiguredo_aom / shiguredo_dav1d / bindgen / shiguredo_cmake / shiguredo_toml) に日本語の用途コメントを追加した
- `ci.yml` の clippy に `--all-targets` を追加した (`-D warnings` は維持)。`Makefile` の clippy ターゲットにも `--all-targets` を追加し、test / cover / check ターゲットにも `--features source-build` を追加して CI と検査条件を統一した
- `prek.toml` に `default_stages = ["pre-commit"]` と `default_install_hook_types = ["pre-commit", "pre-push"]` を追加し、cargo-test に `stages = ["pre-push"]` を追加し、tombi-lint / tombi-format フックを追加した (`Cargo.lock` は除外)。`prek install --prepare-hooks` を再実行して pre-push シムをインストールした
- `CHANGES.md` の develop セクションの misc に [UPDATE] として 3 エントリ追記した (テストメッセージ日本語化 / 依存コメント追加 / 検査設定整備)
