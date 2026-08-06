# 変更履歴

- UPDATE
  - 後方互換がある変更
- ADD
  - 後方互換がある追加
- CHANGE
  - 後方互換のない変更
- FIX
  - バグ修正

## develop

- [CHANGE] MSRV (rust-version) を 1.93 に上げる
  - @voluntas
- [CHANGE] `Encoder::next_frame()` の戻り値を `Option<EncodedFrame>` から `Result<Option<EncodedFrame>, Error>` に変更する
  - @voluntas
- [ADD] Ubuntu 26.04 x86_64 / arm64 向けの prebuilt バイナリを追加する
  - @voluntas
- [ADD] `EncodedFrame::is_keyframe()` の判定対象に Forward Key を追加する
  - @voluntas
- [UPDATE] SVT-AV1 を v4.1.0 から v4.2.0 に更新する
  - @voluntas
- [FIX] `Encoder::encode()` がプレーンサイズの不正を検出しない問題を修正する
  - @voluntas
- [FIX] `DOCS_RS=1` を設定した状態でビルドした後に通常ビルドが失敗する問題を修正する
  - @voluntas

### misc

- [UPDATE] Cargo.toml のフィールド順とセクション順を整理する
  - @voluntas
- [UPDATE] `shiguredo_aom` を 2026.2.0-canary.5 に更新する
  - @voluntas
- [UPDATE] `shiguredo_dav1d` を 2026.2.0-canary.2 に更新する
  - @voluntas
- [UPDATE] `EncoderConfig` の doc コメントを SVT-AV1 v4.2.0 の一次資料と一致するように修正する
  - @voluntas

## 2026.1.0

**リリース日**: 2026-03-31

- [UPDATE] SVT-AV1 を v3.1.2 から v4.1.0 に更新する
  - @voluntas
- [ADD] 8-bit / 10-bit フレームデータを表す `FrameData` enum を追加する
  - @voluntas
- [ADD] H.273 カラー情報の enum を追加する (`ColorPrimaries`, `TransferCharacteristics`, `MatrixCoefficients`, `ColorRange`, `ChromaSamplePosition`)
  - @voluntas
- [ADD] HDR メタデータの struct を追加する (`MasteringDisplayInfo`, `ContentLightLevel`)
  - @voluntas
- [ADD] 品質チューニングの `Tune` enum を追加する (Vq, Psnr, Ssim, Iq, MsSsim)
  - @voluntas
- [ADD] イントラリフレッシュタイプの `IntraRefreshType` enum を追加する (FwdkfRefresh, KfRefresh)
  - @voluntas
- [ADD] フレームタイプの `PictureType` enum を追加する
  - @voluntas
- [ADD] フレームごとのキーフレーム強制用 `EncodeOptions` struct を追加する
  - @voluntas
- [ADD] `Encoder::encode()` でカラーフォーマットとフレームデータの不一致を検出する
  - @voluntas
- [ADD] `Encoder::new()` で設定値のバリデーションを行う
  - @voluntas
- [ADD] `EncoderConfig` に SVT-AV1 v4.0.1 の設定パラメータを大幅に追加する
  - HDR カラー情報 / HDR メタデータ / CBR バッファ設定 / 品質チューニング / S-frame / スーパーレゾリューション / Variance Boost / VBR セクションレート / リサイズ / 量子化マトリクス / フィルター制御 / GOP 制御 / スタートアップ制御 / QP 制御 など
  - @voluntas
- [ADD] `EncodedFrame` にフレーム情報メソッドを追加する (`pts`, `dts`, `temporal_layer_index`, `pic_type`)
  - @voluntas
- [ADD] `EncodedFrame` に品質メトリクスメソッドを追加する (`luma_sse`, `cb_sse`, `cr_sse`, `luma_ssim`, `cb_ssim`, `cr_ssim`, `qp`, `avg_qp`)
  - @voluntas
- [CHANGE] external-dependencies のキー名を `git` から `url` に変更し `.git` サフィックスを削除する
  - @voluntas
- [CHANGE] `ColorFormat` enum のバリアントを `Yuv400`/`Yuv420`/`Yuv422`/`Yuv444` から `I420`/`I42010` に変更する
  - @voluntas
- [CHANGE] `RateControlMode` enum を `RcMode` にリネームする
  - @voluntas
- [CHANGE] `Encoder::encode()` のシグネチャを `encode(y, u, v)` から `encode(frame, options)` に変更する
  - @voluntas
- [CHANGE] `Encoder::new()` の引数を参照 (`&EncoderConfig`) からムーブ (`EncoderConfig`) に変更する
  - @voluntas
- [CHANGE] `Encoder::next_frame()` の戻り値を `Result<Option<EncodedFrame>>` から `Option<EncodedFrame>` に変更する
  - @voluntas
- [CHANGE] `EncoderConfig` から `Default` 実装を削除し `new(width, height, color_format)` コンストラクターに変更する
  - @voluntas
- [CHANGE] `EncoderConfig` のフィールドをリネームする
  - `target_bitrate` → `target_bit_rate`, `encoder_color_format` → `color_format`
  - @voluntas
- [CHANGE] `EncoderConfig` の既存フィールドを `Option` 型に変更する
  - `hierarchical_levels`, `intra_period_length`, `look_ahead_distance`, `over_shoot_pct`, `under_shoot_pct`, `enable_dlf_flag`, `cdef_level`, `enable_restoration_filtering`, `enable_tf`, `enable_overlays`, `film_grain_denoise_strength`, `fast_decode`
  - @voluntas
- [CHANGE] `EncoderConfig` から SVT-AV1 v4.0.1 で不要になったフィールドを削除する
  - `encoder_bit_depth`, `pred_structure`, `pin_threads`, `target_socket`, `enable_tpl_la`, `force_key_frames`, `recon_enabled`, `profile`, `level`, `tier`
  - @voluntas
- [CHANGE] prebuilt バイナリダウンロード機能を追加し `source-build` feature でソースビルドに切り替え可能にする
  - @voluntas
- [CHANGE] source-build を tarball ダウンロードから git clone 方式に変更する
  - @voluntas
- [FIX] CRF モードで `target_bit_rate` を SVT-AV1 に設定しないようにする
  - @voluntas

### misc

- [CHANGE] ビルド依存の `cmake` クレートを `shiguredo_cmake` に置き換える
  - @voluntas
- [CHANGE] ビルド依存の `sha2` クレートを削除し OS コマンドで SHA256 検証する
  - @voluntas
- [CHANGE] ビルド依存の `toml` クレートを `shiguredo_toml` に置き換える
  - @sile

## 2025.1.0

**リリース日**: 2025-09-26
