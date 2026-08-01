# プロジェクト規約違反を是正する

- Created: 2026-08-02
- Completed: (未完了)
- Branch: feature/refactor-fix-rule-violations
- Polished: (未実施)
- Reporter: @voluntas

## 目的

AGENTS.md と shiguredo-rust 規約への違反を是正する。

## 現状

- テストの assert / expect メッセージが英語 (AGENTS.md「テストのログメッセージは全て日本語にすること」違反): `src/lib.rs` のテストと `tests/test_psnr_aom.rs` / `tests/test_psnr_dav1d.rs` の全メッセージ
- Cargo.toml の依存に用途コメントがない (shiguredo-rust 規約「依存ライブラリには用途をコメントで明記すること」違反)
- `.github/workflows/ci.yml` の clippy が `--all-targets` なし (prek.toml との不整合。CI でテストコードが検査されない)
- prek.toml の cargo-test に `pre-push` ステージの指定がない (規約「cargo test は pre-push ステージだけで実行すること」違反)。tombi の lint / format フックもない

## 設計方針

各規約に合わせて修正する。

## 完了条件

- 上記 4 件すべてが規約に適合すること

## 解決方法

- テストの assert / expect メッセージを日本語に変更する
- Cargo.toml の各依存に日本語の用途コメントを追加する
- ci.yml の clippy を `--all-targets --features source-build` に変更する
- prek.toml の cargo-test に `stages = ["pre-push"]` を追加し、tombi フックを追加する
