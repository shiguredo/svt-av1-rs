# Encoder にエンコード途中の動的再設定 (Reconfigure) を追加する

Created: 2026-05-12
Model: Opus 4.7

## 概要

`Encoder::reconfigure(&ReconfigureParams) -> Result<(), Error>` を追加し、エンコーダーを破棄せずにビットレート等を変更できるようにする。姉妹クレート (aom-rs / libvpx-rs / nvcodec-rs / vpl-rs / amf-rs) と同じ API 形を採用する。

## 根拠

姉妹クレート 5 つにはすでに `Encoder::reconfigure` が実装されており、WebRTC / SFU 用途で帯域適応のためにエンコーダーを破棄せず途中でビットレートやフレームレートを切り替えるパターンが共通利用されている。svt-av1-rs だけ未対応のため、呼び出し側で AV1 だけ別パスを書く必要があり扱いにくい。

姉妹クレートでの実装位置:

- aom-rs: `src/lib.rs:1246-1253` (`ReconfigureParams`), `src/lib.rs:1993-2017` (`Encoder::reconfigure`)
- libvpx-rs: `src/lib.rs:698-738`, `src/lib.rs:1447-1499`, `src/lib.rs:818-918` (検査ヘルパ `merge_reconfigure_params_into_cfg`)
- nvcodec-rs: `src/encode.rs:399-414`, `src/encode.rs:652-738`
- vpl-rs: `src/encode.rs:425-434`, `src/encode.rs:805-833`
- amf-rs: `src/encode.rs:246-255`, `src/encode.rs:460-483`

## SVT-AV1 ネイティブ API における制約 (重要)

SVT-AV1 v4.1.0 のヘッダ (`EbSvtAv1Enc.h`) では API の呼び出し順序が **STEP 1〜7** として明示されており、`svt_av1_enc_set_parameter` は STEP 2 (init 前) に 1 回だけ呼ぶ前提になっている:

```
STEP 1: svt_av1_enc_init_handle
STEP 2: svt_av1_enc_set_parameter  ← パラメータはここで全部設定
STEP 3: svt_av1_enc_init           ← init 後の再設定は想定されていない
STEP 4: svt_av1_enc_send_picture
STEP 5: svt_av1_enc_get_packet
STEP 6: svt_av1_enc_deinit
STEP 7: svt_av1_enc_deinit_handle
```

実装 (`Source/Lib/Globals/enc_handle.c:4434` の `svt_av1_enc_set_parameter`) を読むと、内部で `prediction_structure_group_ptr` を `EB_NO_THROW_NEW` で **新規確保** し、`load_default_buffer_configuration_settings` を呼んでいる。これは `svt_av1_enc_init` の前提とする「config 初期化フェーズ」の処理であり、init 後に再度呼ぶと:

- `prediction_structure_group_ptr` の二重確保 (リーク)
- 既に確保済みのバッファとの不整合
- `SequenceControlSet` のフィールド書き換えによる動作中スレッドへの影響

が起きうる。**aom-rs / libvpx-rs の `aom_codec_enc_config_set` / `vpx_codec_enc_config_set` のような「init 後に config を差し替える専用 API」は SVT-AV1 にはない**。`svt_av1_enc_set_parameter` の単純な再呼び出しでは安全に動的再設定はできない。

一方、SVT-AV1 にも以下のような **事前指定型** または **per-picture 制御** の動的機構は存在する:

- `EbSvtAv1EncConfiguration::force_key_frames` (`EbSvtAv1Enc.h:709-712`) — 初期化時にフラグを立てておき、`EbBufferHeaderType::pic_type` を `EB_AV1_KEY_PICTURE` にして送ると当該フレームがキーフレームになる (本クレートでは既に `EncodeOptions::force_keyframe` として実装済み、`src/lib.rs:1396-1400`)
- `SvtAv1FrameScaleEvts` (`EbSvtAv1Enc.h:188` 付近) — フレーム番号と解像度変更を **事前に** 指定する仕組み
- `sframe_posi` — S-frame の位置を事前指定する仕組み

これらは「init 時に未来の変更スケジュールを渡しておく」型であり、aom-rs のような「任意のタイミングで `reconfigure` を呼んで反映」とは性質が異なる。

## 確認事項 (issue 起票時点で未解決)

機能着手前に以下を確定させる必要がある。優先度順:

1. **SVT-AV1 が `svt_av1_enc_set_parameter` の init 後再呼び出しを公式にサポートしているか**
   - 一次情報: SVT-AV1 GitLab (gitlab.com/AOMediaCodec/SVT-AV1) の issue / MR / docs/
   - 公式 docs に "dynamic" / "reconfigure" / "bitrate change at runtime" の記述があるか
   - 既存 issue / discussion を検索する
2. **再呼び出しが許される場合、どのフィールドが安全に変更できるか**
   - `target_bit_rate` / `max_bit_rate` / `min_qp_allowed` / `max_qp_allowed` / `intra_period_length` / `enc_mode` / `frame_rate_numerator` / `frame_rate_denominator`
   - 不変として扱うべきもの: `source_width` / `source_height` / `encoder_bit_depth` / `encoder_color_format` / `profile` / `force_key_frames` (init 時固定の flag)
3. **再呼び出しを許さない場合の代替案**
   - per-picture metadata (`EbBufferHeaderType` の qp / pic_type) による疑似的な動的制御で代用するか
   - `Encoder` を破棄・再生成する高水準ヘルパを提供するか
   - issue を `pending/` に移動して見送るか

上記が確定するまでは実装に入らない。

## 想定する API

姉妹クレートとの API 統一を最優先とし、最小フィールドから始める。

```rust
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ReconfigureParams {
    /// ターゲットビットレート (bps)。SVT-AV1 の `target_bit_rate` (u32) に対応
    pub target_bitrate: Option<u32>,

    /// 最大ビットレート (bps)。SVT-AV1 の `max_bit_rate` (u32) に対応
    pub max_bitrate: Option<u32>,

    /// 最小 QP (0-63)。SVT-AV1 の `min_qp_allowed` に対応
    pub min_qp_allowed: Option<u8>,

    /// 最大 QP (0-63)。SVT-AV1 の `max_qp_allowed` に対応
    pub max_qp_allowed: Option<u8>,

    /// キーフレーム最大間隔 (フレーム数)。SVT-AV1 の `intra_period_length` に対応
    pub intra_period_length: Option<NonZeroUsize>,

    /// FPS 分子。`frame_rate_denominator` と必ず同時指定
    pub frame_rate_numerator: Option<u32>,

    /// FPS 分母。`frame_rate_numerator` と必ず同時指定
    pub frame_rate_denominator: Option<u32>,
}

impl Encoder {
    pub fn reconfigure(&mut self, params: &ReconfigureParams) -> Result<(), Error>;
}
```

API 設計方針:

- 全フィールド `Option<T>`。`None` は「現在値を維持」
- `#[non_exhaustive]` を付ける (libvpx-rs と同じ)
- ビットレート単位は SVT-AV1 ネイティブと揃え bps (u32)。aom-rs は kbps だが SVT-AV1 の `target_bit_rate` が bps なので bps に倒す
- 不変として扱うフィールド: `source_width` / `source_height` / `encoder_color_format` / `encoder_bit_depth` / `profile` / `enc_mode` (`enc_mode` は SVT-AV1 内部で多くの設定を派生決定するため動的変更は危険)
- `force_keyframe` は既に `EncodeOptions::force_keyframe` が存在するので `ReconfigureParams` には入れない

## 検査ロジック

libvpx-rs の `merge_reconfigure_params_into_cfg` (`src/lib.rs:818-918`) と同じパターンを採用し、`Encoder::new` と `Encoder::reconfigure` で共通のヘルパに集約する。これにより両者の検査ルールがズレない。

検査内容:

- `target_bitrate`: 0 以外。SVT-AV1 が許容する上限値を確認のうえ反映
- `max_bitrate`: `target_bitrate <= max_bitrate` (両方の現在値も含めて検査)
- `min_qp_allowed` / `max_qp_allowed`: 0-63 範囲、`min <= max` (libvpx-rs と同じく現在値とも比較)
- `frame_rate_numerator` / `frame_rate_denominator`: 両方同時指定必須、非ゼロ、SVT-AV1 上限内
- `intra_period_length`: `NonZeroUsize` で 0 を型で弾く

検査失敗時は SVT-AV1 を呼ばずに即エラー、内部状態は変更しない (トランザクション性)。

## 呼び出し順序の制約

aom-rs / libvpx-rs 同様、`encode()` → `next_frame()` のドレイン中 (= 内部にまだパケットが残っている状態) では `reconfigure()` を拒否する。本クレートの場合は `received_count < frame_count` 等で「未回収パケット有り」を判定する必要がある (要確認: SVT-AV1 は明示的な iterator がないため、判定方法を実装時に詰める)。

EOS 送信後 (`self.eos == true`) の `reconfigure` も拒否する。

## CHANGES.md エントリ案

```
- [ADD] `Encoder::reconfigure` と `ReconfigureParams` を追加する
  - エンコード途中でビットレート・QP 範囲・キーフレーム間隔・フレームレートを変更できる
  - 値域外はすべて `INVALID_PARAM` で拒否する
  - 失敗時は内部設定を変更しない
  - @voluntas
```

## テスト方針

aom-rs / libvpx-rs を参考に最低限以下をカバーする:

- midstream でビットレートを変更し、変更前後で出力バイト数比に有意差が出ることを確認
- 全フィールド `None` の `reconfigure` が no-op として成功し、ネイティブ API を呼ばない
- 値域外 (QP 64、bitrate 0、片方だけの fps、min > max など) を全て拒否
- `reconfigure` 失敗時に内部状態がロールバックされること (失敗後に妥当な値で再 `reconfigure` できる)
- `reconfigure` 後の `force_keyframe` 併用で実際にキーフレームが挿入されること
- EOS 後の `reconfigure` が拒否されること

## 関連 issue / 参考

- aom-rs: 0004-0012 の連番 issue で段階的に育てている (検査ロジック共通化 / FFI ロールバック / iter ガード / テスト拡充 / example 追加)
- libvpx-rs: silent clip 完全排除方針で `Encoder::new` も同時に検査強化された
- 本クレートでは `Encoder::new` の検査ロジックも `reconfigure` 用ヘルパに合わせて再点検する (libvpx-rs と同じ抱き合わせ変更が必要になる可能性が高い)
