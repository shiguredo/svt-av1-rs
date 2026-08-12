# Encoder::next_frame() のエラー処理と早期リターンを改善する

- Created: 2026-08-02
- Completed: 2026-08-06
- Branch: feature/change-next-frame-error-handling
- Polished: 2026-08-06
- Reporter: @voluntas

## 目的

エンコードエラーの握りつぶしをなくし、`next_frame()` の前提条件を実装とドキュメントの両方で正す。

## 現状

- `Encoder::next_frame()` は `EB_NoErrorEmptyQueue` 以外の FFI エラーも `None` として返し、正常終了と区別できない。不完全なビットストリームがユーザーに無音で渡る (API の戻り値としては検出できない。ログは `log::error!` で出力される)
- `received_count >= frame_count` による早期リターンは「出力パケット数 == 入力フレーム数」を暗黙に前提している。ARF / overlay パケットが出力される構成 (VBR / CRF の RANDOM_ACCESS では ARF が生成され得る) では、EOS 前の drain が不完全になる
- 実機で確認済み: VBR (RANDOM_ACCESS) では `finish()` を呼ぶまでパケットが 1 つも返らない (CRF も VBR と同一の pred_structure 構成のため同じ挙動と推測される)。この挙動は公開ドキュメントに記載がない

## 設計方針

- `Encoder::next_frame()` の戻り値を `Result<Option<EncodedFrame>, Error>` に変更し、FFI エラーを伝播させる。正常系の `EB_NoErrorEmptyQueue` / `EB_NoErrorFifoShutdown` と、EOS フラグ付きパケットを受け取った場合と、早期リターンは `Ok(None)` を返す。それ以外のエラーは `Err` とする。成功コードで output が null の場合は `Err` とする (code は既存の null ガードと同様に `EB_ErrorBadParameter` を使用する。2026.1.0 で戻り値を Option に簡素化したが、FFI エラーの握りつぶしという副作用があったため逆方向に変更する)
- 早期リターンを CBR (LOW_DELAY) でのみ有効にする (パケット数ベースの判定は ARF / overlay パケット数の見積もりがコンテンツ依存で不可能なため不採用)。早期リターンは元々 CBR の `svt_av1_enc_get_packet` ブロッキング対策として導入されたものであり、CBR 限定は元の意図への回帰。CBR は pred_structure=1 (LOW_DELAY) が強制され、LOW_DELAY では ARF / overlay が生成されないため出力数 == 入力数が保証される。VBR / CRF では EOS 送信前は `svt_av1_enc_get_packet` は非ブロッキングで `EB_NoErrorEmptyQueue` を返す
- VBR / CRF の既定構成 (overlay なし) で `finish()` までパケットが得られないことを `encode()` と `next_frame()` の doc に明記する

## 完了条件

- FFI エラーが `Err` として呼び出し元に伝播すること (`EB_NoErrorEmptyQueue` / `EB_NoErrorFifoShutdown` は `Ok(None)`)
- 早期リターンが CBR (LOW_DELAY) でのみ働くこと (既存の PSNR CBR テストが encode 直後の drain でこの経路を踏む。VBR / CRF では早期リターンが発動しないことはコード確認で担保する。早期リターンの `Ok(None)` と `EB_NoErrorEmptyQueue` の `Ok(None)` は戻り値で区別できないためテスト化は困難)
- パケット遅延の挙動がドキュメントに記載されること (`encode()` / `next_frame()` の doc。README はサンプルコードのコンパイルのみが対象)
- 既存テストがすべて成功すること (src/lib.rs の `#[cfg(test)]` モジュールと `tests/` の PSNR テストは Result 化に追随して書き換える)
- `CHANGES.md` の develop セクションに [CHANGE] として追記すること
- `README.md` のサンプルコードが新しいシグネチャでコンパイル可能なこと

## 解決方法

- `src/lib.rs` の `Encoder::next_frame()` を `Result<Option<EncodedFrame>, Error>` に変更した (公開 API の破壊的変更のため、`CHANGES.md` に [CHANGE] として記載した)
  - FFI エラーは `Err` として呼び出し元に伝播する。`EB_NoErrorEmptyQueue` / `EB_NoErrorFifoShutdown`・EOS フラグ付きパケット・早期リターンは `Ok(None)` を返す
  - 成功コードで null が返った場合は `EB_ErrorBadParameter` の `Err` を返す
  - FFI エラー時に SVT-AV1 がセットするエラーパケットは `svt_av1_enc_release_out_buffer` でバッファプールへ返却してから `Err` を返す (リーク防止)
- `Encoder` 構造体に `rate_control_mode` フィールドを追加し、早期リターンを CBR (LOW_DELAY) でのみ有効にした
- CBR と FwdkfRefresh の組み合わせは、LOW_DELAY でパケット生成が入力フレーム数に追いつかず `next_frame()` が永久ブロックすることを実測確認し、`validate_config` で禁止した (テスト `cbr_fwdkf_refresh_rejected` を追加)
- `encode()` / `next_frame()` の doc に、VBR / CRF の既定構成 (overlay なし) では `finish()` を呼ぶまでパケットが得られないことがある旨を明記した (原因は RANDOM_ACCESS のルックアヘッドによるパケット生成の遅延)
- 呼び出し側を追随して書き換えた (`src/lib.rs` の `#[cfg(test)]` モジュールの encode 系テスト、`tests/test_psnr_aom.rs` / `tests/test_psnr_dav1d.rs`、`README.md` のサンプルコード)
- `encode_cbr` を encode 直後の drain パターンに変更し、早期リターン経路を単体テストで検証する回帰ガードとした (PSNR CBR テストにも早期リターン回帰ガードのコメントを追記)
- `CHANGES.md` の develop セクションに [CHANGE] として 2 エントリ追記した (next_frame() の Result 化、CBR と FwdkfRefresh の組み合わせ禁止)
