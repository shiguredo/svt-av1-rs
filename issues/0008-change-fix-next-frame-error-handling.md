# Encoder::next_frame() のエラー処理と早期リターンを改善する

- Created: 2026-08-02
- Completed: (未完了)
- Branch: feature/change-next-frame-error-handling
- Polished: (未実施)
- Reporter: @voluntas

## 目的

エンコードエラーの握りつぶしをなくし、`next_frame()` の前提条件を実装とドキュメントの両方で正す。

## 現状

- `Encoder::next_frame()` は `EB_NoErrorEmptyQueue` 以外の FFI エラーも `None` として返し、正常終了と区別できない。不完全なビットストリームがユーザーに無音で渡る
- `received_count >= frame_count` による早期リターンは「出力パケット数 == 入力フレーム数」を暗黙に前提している。ARF / overlay パケットが出力される構成では、EOS 前の drain が不完全になる
- 実機で確認済み: VBR / CRF (RANDOM_ACCESS) では `finish()` を呼ぶまでパケットが 1 つも返らない。この挙動は公開ドキュメントに記載がない

## 設計方針

- `Encoder::next_frame()` の戻り値を `Result<Option<EncodedFrame>, Error>` に変更し、FFI エラーを伝播させる
- 早期リターンを CBR (LOW_DELAY) でのみ有効にするか、パケット数ベースの判定に見直す
- VBR / CRF で `finish()` までパケットが得られないことを `encode()` と `next_frame()` の doc に明記する

## 完了条件

- FFI エラーが `Err` として呼び出し元に伝播すること
- 早期リターンが正しいモードでのみ働くこと
- パケット遅延の挙動がドキュメントに記載されること
- 既存テスト (encode_black / encode_cbr / encode_crf / PSNR テスト) がすべて成功すること

## 解決方法

- `src/lib.rs` の `Encoder::next_frame()` を `Result<Option<EncodedFrame>, Error>` に変更する (公開 API の破壊的変更のため、`CHANGES.md` に [CHANGE] として記載する)
- 早期リターンの条件を CBR に限定する
- `encode()` / `next_frame()` の doc を更新する
