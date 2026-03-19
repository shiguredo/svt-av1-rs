# svt-av1-rs

[![crates.io](https://img.shields.io/crates/v/shiguredo_svt_av1.svg)](https://crates.io/crates/shiguredo_svt_av1)
[![docs.rs](https://docs.rs/shiguredo_svt_av1/badge.svg)](https://docs.rs/shiguredo_svt_av1)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![GitHub Actions](https://github.com/shiguredo/svt-av1-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/shiguredo/svt-av1-rs/actions/workflows/ci.yml)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?logo=discord&logoColor=white)](https://discord.gg/shiguredo)

## About Shiguredo's open source software

We will not respond to PRs or issues that have not been discussed on Discord. Also, Discord is only available in Japanese.

Please read <https://github.com/shiguredo/oss> before use.

## 時雨堂のオープンソースソフトウェアについて

利用前に <https://github.com/shiguredo/oss> をお読みください。

## shiguredo_svt_av1 について

[SVT-AV1](https://gitlab.com/AOMediaCodec/SVT-AV1) を利用した AV1 エンコーダーの Rust バインディングです。

## 特徴

- AV1 エンコーダー
- I420 (YUV 4:2:0 8-bit) / I42010 (YUV 4:2:0 10-bit) 入力対応
- HDR 対応 (BT.2020, PQ, HLG, マスタリングディスプレイ情報, Content Light Level)
- エンコーダーの詳細設定 (レート制御、品質、速度)
- per-frame のキーフレーム強制
- per-frame の品質メトリクス (PSNR, SSIM)
- 品質チューニング (VQ, PSNR, SSIM, IQ, MS-SSIM)
- フィルムグレインシンセシス
- スクリーンコンテンツモード
- RTC (Real-Time Coding) モード
- タイル分割による並列処理
- AV1 インループフィルター制御 (デブロッキング、CDEF、リストレーション)
- テンポラルフィルター / Alt-Ref フレーム制御
- スーパーレゾリューション / 動的リサイズ
- S-frame (スイッチフレーム) サポート
- ロスレスエンコード / AVIF 静止画モード
- Variance Boost
- 量子化マトリクス
- データ駆動型 GOP
- prebuilt バイナリによる高速ビルド (デフォルト)
- ソースからのビルドも可能 (`--features source-build`)

## 動作要件

- Ubuntu 24.04 x86_64
- Ubuntu 24.04 arm64
- Ubuntu 22.04 x86_64
- Ubuntu 22.04 arm64
- macOS 26 arm64
- macOS 15 arm64
- Windows 11 x86_64
- Windows Server 2025 x86_64

### ソースビルド時の追加要件

- Git
- C コンパイラ (`build-essential` 等)
- NASM (SVT-AV1 のアセンブリ最適化に必要)

```bash
# Ubuntu
sudo apt-get install -y build-essential nasm

# macOS
brew install nasm

# Windows
choco install nasm
```

## ビルド

デフォルトでは GitHub Releases から prebuilt バイナリをダウンロードしてビルドします。

```bash
cargo build
```

### ソースからビルド

SVT-AV1 をソースからビルドする場合は `source-build` feature を有効にしてください。

```bash
cargo build --features source-build
```

### docs.rs 向けビルド

SVT-AV1 がない環境では、docs.rs 向けのドキュメント生成のみ可能です。

```bash
DOCS_RS=1 cargo doc --no-deps
```

## 使い方

### エンコード

入力は `FrameData` 列挙型で画像フォーマットと各プレーンのデータを渡します。

```rust
use shiguredo_svt_av1::{
    ColorFormat, EncodeOptions, Encoder, EncoderConfig, FrameData, RcMode,
};

// 必須パラメータを指定して設定を生成
let mut config = EncoderConfig::new(
    1920,              // width
    1080,              // height
    ColorFormat::I420, // color_format
);

// 必要に応じてオプションパラメータを変更
config.rate_control_mode = RcMode::Cbr;
config.target_bit_rate = 4_000_000;
config.enc_mode = 8; // 0-13 (0=最高品質, 13=最速)

// エンコーダーを作成
let mut encoder = Encoder::new(config)?;

// I420 形式の YUV データをエンコード
let frame = FrameData::I420 { y: &y_data, u: &u_data, v: &v_data };
encoder.encode(&frame, &EncodeOptions::default())?;

// キーフレームを強制する場合
encoder.encode(&frame, &EncodeOptions { force_keyframe: true })?;

// エンコード済みフレームを取得
while let Some(frame) = encoder.next_frame() {
    let data = frame.data();
    let is_key = frame.is_keyframe();
    println!("encoded: {} bytes, keyframe: {}", data.len(), is_key);
}

// 残りのフレームをフラッシュ
encoder.finish()?;
while let Some(frame) = encoder.next_frame() {
    // ...
}
```

## 設定

### `EncoderConfig`

#### 基本設定

| フィールド | 型 | 説明 |
|---|---|---|
| `width` | `usize` | 映像の幅 |
| `height` | `usize` | 映像の高さ |
| `color_format` | `ColorFormat` | 入力画像フォーマット |
| `fps_numerator` | `usize` | フレームレートの分子 |
| `fps_denominator` | `usize` | フレームレートの分母 |
| `target_bit_rate` | `usize` | ターゲットビットレート (bps) |
| `rate_control_mode` | `RcMode` | レート制御モード |
| `enc_mode` | `u8` | エンコードプリセット (0-13) |
| `scene_change_detection` | `bool` | シーンチェンジ検出 |
| `stat_report` | `bool` | per-frame の品質メトリクス計算を有効にする |

#### 量子化

| フィールド | 型 | 説明 |
|---|---|---|
| `min_qp_allowed` | `Option<u8>` | 最小量子化パラメータ (0-63) |
| `max_qp_allowed` | `Option<u8>` | 最大量子化パラメータ (0-63) |
| `qp` | `Option<u8>` | 量子化パラメータ (0-63, CRF モード時) |
| `aq_mode` | `Option<u8>` | 適応的量子化モード (0=CQP, 1=variance, 2=CRF) |
| `enable_qm` | `Option<bool>` | 量子化マトリクスの有効化 |
| `min_qm_level` | `Option<u8>` | 最小 QM レベル (0-15) |
| `max_qm_level` | `Option<u8>` | 最大 QM レベル (0-15) |
| `min_chroma_qm_level` | `Option<u8>` | クロマ用最小 QM レベル (0-15) |
| `max_chroma_qm_level` | `Option<u8>` | クロマ用最大 QM レベル (0-15) |

#### GOP / フレーム構造

| フィールド | 型 | 説明 |
|---|---|---|
| `intra_period_length` | `Option<NonZeroUsize>` | キーフレーム間隔 |
| `intra_refresh_type` | `Option<IntraRefreshType>` | イントラリフレッシュタイプ |
| `hierarchical_levels` | `Option<u32>` | テンポラルレイヤーの階層数 (0-5) |
| `look_ahead_distance` | `Option<usize>` | 先読み距離 (フレーム数) |
| `enable_dg` | `Option<bool>` | データ駆動型 GOP |
| `gop_constraint_rc` | `Option<bool>` | GOP 単位レート制御制約 |
| `multiply_keyint` | `Option<bool>` | キーフレーム間隔を秒単位として扱う |

#### レート制御

| フィールド | 型 | 説明 |
|---|---|---|
| `max_bit_rate` | `Option<usize>` | 最大ビットレート (bps, Capped CRF 用) |
| `vbr_min_section_pct` | `Option<u32>` | VBR 最小セクションレート (%) |
| `vbr_max_section_pct` | `Option<u32>` | VBR 最大セクションレート (%) |
| `under_shoot_pct` | `Option<u32>` | アンダーシュート許容割合 |
| `over_shoot_pct` | `Option<u32>` | オーバーシュート許容割合 |
| `mbr_over_shoot_pct` | `Option<u32>` | 最大ビットレート オーバーシュート許容割合 |
| `recode_loop` | `Option<u32>` | リコードループ (0=無効, 1=キーフレームのみ, 2=全フレーム) |
| `starting_buffer_level_ms` | `Option<u64>` | CBR 初期バッファレベル (ms) |
| `optimal_buffer_level_ms` | `Option<u64>` | CBR 目標バッファレベル (ms) |
| `maximum_buffer_size_ms` | `Option<u64>` | CBR 最大バッファサイズ (ms) |

#### 並列処理

| フィールド | 型 | 説明 |
|---|---|---|
| `tile_columns` | `Option<NonZeroUsize>` | タイル列数 |
| `tile_rows` | `Option<NonZeroUsize>` | タイル行数 |
| `level_of_parallelism` | `Option<u32>` | 並列化レベル |

#### カラー情報

| フィールド | 型 | 説明 |
|---|---|---|
| `color_primaries` | `Option<ColorPrimaries>` | カラープライマリ |
| `transfer_characteristics` | `Option<TransferCharacteristics>` | 伝達特性 |
| `matrix_coefficients` | `Option<MatrixCoefficients>` | マトリクス係数 |
| `color_range` | `Option<ColorRange>` | カラーレンジ |
| `chroma_sample_position` | `Option<ChromaSamplePosition>` | クロマサンプル位置 |
| `mastering_display` | `Option<MasteringDisplayInfo>` | HDR マスタリングディスプレイ情報 |
| `content_light_level` | `Option<ContentLightLevel>` | HDR コンテンツ輝度レベル |

#### 品質チューニング

| フィールド | 型 | 説明 |
|---|---|---|
| `tune` | `Option<Tune>` | 品質チューニングメトリクス |
| `sharpness` | `Option<i8>` | シャープネス (-7 to 7) |
| `enable_variance_boost` | `Option<bool>` | Variance Boost の有効化 |
| `variance_boost_strength` | `Option<u8>` | Variance Boost 強度 |
| `variance_octile` | `Option<u8>` | 分散オクタイル |
| `variance_boost_curve` | `Option<u8>` | Variance Boost カーブ |
| `luminance_qp_bias` | `Option<u8>` | 輝度 QP バイアス |
| `qp_scale_compress_strength` | `Option<u8>` | QP スケール圧縮強度 |
| `extended_crf_qindex_offset` | `Option<u8>` | 拡張 CRF QIndex オフセット |
| `ac_bias` | `Option<f64>` | 高周波誤差バイアス (テクスチャ保持) |

#### フィルター制御

| フィールド | 型 | 説明 |
|---|---|---|
| `enable_dlf_flag` | `Option<u8>` | デブロッキングループフィルター (0=無効, 1=有効, 2=高精度) |
| `cdef_level` | `Option<i32>` | CDEF レベル (-1=自動, 0=無効, 1-5=レベル) |
| `enable_restoration_filtering` | `Option<i32>` | リストレーションフィルタリング (-1=自動, 0=無効, 1=有効) |
| `enable_tf` | `Option<u8>` | テンポラルフィルター (0=無効, 1=有効, 2=適応的) |
| `tf_strength` | `Option<u8>` | テンポラルフィルター強度 |
| `enable_overlays` | `Option<bool>` | オーバーレイフレーム有効化 |
| `enable_mfmv` | `Option<i32>` | Motion Field Motion Vector (-1=自動, 0=無効, 1=有効) |

#### エンコード制御

| フィールド | 型 | 説明 |
|---|---|---|
| `fast_decode` | `Option<u8>` | デコーダー速度最適化 (0-2) |
| `screen_content_mode` | `Option<u8>` | スクリーンコンテンツモード (0-3) |
| `rtc` | `Option<bool>` | RTC モード |
| `lossless` | `Option<bool>` | ロスレスエンコード |
| `avif` | `Option<bool>` | AVIF 静止画エンコードモード |
| `max_tx_size` | `Option<u8>` | 最大トランスフォームサイズ |
| `film_grain_denoise_strength` | `Option<u32>` | フィルムグレインデノイズ強度 (0-50) |
| `film_grain_denoise_apply` | `Option<u8>` | フィルムグレインデノイズ適用 (0-1) |
| `adaptive_film_grain` | `Option<bool>` | 適応的フィルムグレインブロックサイズ |

#### スーパーレゾリューション / リサイズ

| フィールド | 型 | 説明 |
|---|---|---|
| `superres_mode` | `Option<u8>` | スーパーレゾリューションモード (0-4) |
| `superres_denom` | `Option<u8>` | ダウンスケール分母 (8-16) |
| `superres_kf_denom` | `Option<u8>` | キーフレーム用ダウンスケール分母 (8-16) |
| `superres_qthres` | `Option<u8>` | QThreshold モードの閾値 |
| `superres_kf_qthres` | `Option<u8>` | キーフレーム用 QThreshold |
| `superres_auto_search_type` | `Option<u8>` | 自動検索タイプ |
| `resize_mode` | `Option<u8>` | リサイズモード (0-3) |
| `resize_denom` | `Option<u8>` | リサイズ分母 (8-16) |
| `resize_kf_denom` | `Option<u8>` | キーフレーム用リサイズ分母 (8-16) |

#### S-frame

| フィールド | 型 | 説明 |
|---|---|---|
| `sframe_dist` | `Option<i32>` | S-frame 挿入間隔 (フレーム数) |
| `sframe_mode` | `Option<u32>` | S-frame 挿入モード (1=STRICT, 2=NEAREST) |
| `sframe_qp` | `Option<u8>` | S-frame の QP 値 |
| `sframe_qp_offset` | `Option<i8>` | S-frame の QP オフセット |

#### スタートアップ制御

| フィールド | 型 | 説明 |
|---|---|---|
| `startup_mg_size` | `Option<u8>` | スタートアップ MG サイズ |
| `startup_qp_offset` | `Option<i8>` | スタートアップ GOP の QP オフセット |

### `RcMode`

| バリアント | 説明 |
|---|---|
| `CqpOrCrf` | CQP/CRF (品質ベース) |
| `Vbr` | Variable Bitrate (可変ビットレート) |
| `Cbr` | Constant Bitrate (固定ビットレート) |

### `ColorFormat`

| バリアント | 説明 |
|---|---|
| `I420` | YUV 4:2:0 planar 8-bit (3 プレーン: Y, U, V) |
| `I42010` | YUV 4:2:0 planar 10-bit (3 プレーン: Y, U, V、各ピクセル 2 バイト) |

### `FrameData`

| バリアント | 説明 |
|---|---|
| `I420 { y, u, v }` | I420 (3 プレーン: Y, U, V) |
| `I42010 { y, u, v }` | I42010 (3 プレーン: Y, U, V、各ピクセル 2 バイト) |

### `Tune`

| バリアント | 説明 |
|---|---|
| `Vq` | VQ (Visual Quality) |
| `Psnr` | PSNR |
| `Ssim` | SSIM |
| `Iq` | IQ (Image Quality) — v4.0.0 以降 |
| `MsSsim` | MS-SSIM |

### `IntraRefreshType`

| バリアント | 説明 |
|---|---|
| `FwdkfRefresh` | Forward Key Frame Refresh (Open GOP) |
| `KfRefresh` | Key Frame Refresh (Closed GOP) |

### `EncodeOptions`

| フィールド | 型 | デフォルト | 説明 |
|---|---|---|---|
| `force_keyframe` | `bool` | false | キーフレームを強制する |

## SVT-AV1 ライセンス

<https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/master/LICENSE.md>

```text
BSD 3-Clause Clear License

Copyright (c) 2019, Alliance for Open Media
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice,
   this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
POSSIBILITY OF SUCH DAMAGE.
```

## ライセンス

Apache License 2.0

```text
Copyright 2026-2026, Shiguredo Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
