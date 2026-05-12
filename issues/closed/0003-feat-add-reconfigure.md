# Encoder::reconfigure の追加可否を調査する

Created: 2026-05-12
Completed: 2026-05-12
Model: Opus 4.7

## 概要

姉妹クレート (aom-rs / libvpx-rs / nvcodec-rs / vpl-rs / amf-rs) では `Encoder::reconfigure` が実装されているが、svt-av1-rs には存在しない。本クレートでも提供できるかを SVT-AV1 v4.1.0 タグの公開 API で一次情報から裏取りする。本クレートが追従するバージョンは `Cargo.toml` の `[package.metadata.external-dependencies]` で `v4.1.0` に固定されている。

## 判定結果の定義

本文中はこの 3 語のいずれかのみを使う。「判定 = 方針」と扱い、確定した時点で対応する処理フローへ移る。

- **サポートあり**: SVT-AV1 ネイティブの公開 API で、init 後にビットレートを含む 1 つ以上のフィールドの動的変更を行う正規の手順が一次情報で確認できる。本 issue を `issues/closed/` へ移し、後続の実装 issue を新規起票する。
- **部分サポート**: 正規の手順は確認できないが、ピクチャー単位 (per-picture) 制御 (`EbBufferHeaderType` 経由) で本クレートが必要とする動的変更 (ピクチャー単位 QP 等) の一部が達成できる。スコープを再定義した別 issue を起票し、本 issue を `issues/closed/` へ移す。
- **サポートなし**: 上記いずれも確認できない。本 issue を `issues/closed/` へ移す。

closed への移動はいずれの場合も、確認事項で集めた一次情報 (commit hash・permalink・URL) と各確認事項の判定 (および後続 issue 番号があればその番号) を「## 解決方法」セクションに追記してから次のコマンドで実行する。

```
git mv issues/0003-feat-add-reconfigure.md issues/closed/0003-feat-add-reconfigure.md
```

CLAUDE.md「### 設計判断が必要な issue の場合」に該当するのは、確認事項を尽くしても v4.1.0 タグの挙動が一次情報で結論できないと判明した場合に限る。その場合のみ `issues/closed/` ではなく `issues/pending/` へ移動し、pending の理由を追記する。

## 根拠

WebRTC / SFU 用途では帯域適応のためエンコーダーを破棄せず途中でビットレートやフレームレートを切り替える運用パターンがあり、姉妹クレートはこれを `Encoder::reconfigure` として提供している。本クレートだけ未対応だと呼び出し側で AV1 のみ別パスを書くことになり扱いにくい。

直近のリリース 2026.1.0 で SVT-AV1 を v3.1.2 から v4.1.0 タグへ追従し公開 API が大幅に変動したため、このタイミングで一次情報から評価する。

## 確認事項

以下を一次情報で裏取りする。**確認事項 1 が「サポートあり」と確定した場合でも、`svt_av1_enc_set_parameter` の再呼び出しでランタイム反映されないフィールドが残る可能性があるため、確認事項 2 もそのまま実施する**。確認事項 1 で「サポートなし」が確定した場合は確認事項 2 で **部分サポート** を判定する。

### 共通の調査手順

- SVT-AV1 v4.1.0 タグの commit hash を SVT-AV1 GitLab (`https://gitlab.com/AOMediaCodec/SVT-AV1`) で取得し、調査記録に残す
- 引用するヘッダ / 実装ファイルは v4.1.0 タグの permalink で参照する

### 確認事項 1: init 後の `svt_av1_enc_set_parameter` 再呼び出しが公式にサポートされているか

調査対象:

- `Source/API/EbSvtAv1Enc.h` の `svt_av1_enc_set_parameter` のドキュメントコメント全文 (STEP 2 表記の周辺)
- `Source/Lib/Globals/enc_handle.c` の `svt_av1_enc_set_parameter` 実装 (init 後再呼び出し時のリソース確保・解放挙動、`init_done` 相当の state チェック分岐の有無、二重解放 / リーク経路の有無、init 後再呼び出しで実際にランタイム反映されるフィールドと反映されないフィールドの一覧)
- `Source/Lib/Globals/enc_settings.c` の `svt_av1_verify_settings` (init 前提のチェックが含まれるか)

判定基準: **サポートあり** とするには以下を **両方** 満たすこと。

- `Source/API/EbSvtAv1Enc.h` のドキュメントコメントと公式ドキュメント (`Docs/` 配下、Encoder User Guide 等) の双方を確認し、双方が init 後再呼び出しを許容する旨を示していること。片方のみの言及、または双方の記述が矛盾している場合は調査結果に記録のうえ **サポートなし** 扱い
- `Source/Lib/Globals/enc_handle.c` の実装で init 後再呼び出し時にリソース二重確保 / リーク / state 不整合が発生しないコード経路が確認できること

公式 sample app (`SvtAv1EncApp`、ソースは `Source/App/` 配下) で init 後再呼び出しを行うコード経路の有無は補強材料として記録するが、判定の必須条件ではない (sample app は CLI 用途中心でランタイム再設定をカバーしない可能性が高いため)。

GitLab Issues / Merge Requests (以下 Issue / MR) で議論はあるが v4.1.0 タグにマージされていない場合は **サポートなし** 扱いとする。

### 確認事項 2: ピクチャー単位制御 / 個別フィールド変更 API の有無

調査対象:

- `Source/API/EbSvtAv1Enc.h` の `OPTIONAL` コメント付き関数群のうち、init 済みハンドル (`EbComponentType *`) を引数に取るもののドキュメントコメントとシグネチャ
- `Source/API/EbSvtAv1.h` の `EbBufferHeaderType` 全フィールド (`pic_type` / `qp` 等) の意味とランタイム影響
- `EbBufferHeaderType::qp` が SVT-AV1 内部でピクチャー単位 QP として実際に使われる経路。`Source/Lib/Codec/resource_coordination_process.c` の `qp_on_the_fly` 分岐と、その前提条件である公開フラグ `EbSvtAv1EncConfiguration::use_qp_file` の挙動。`use_qp_file=true` が必須かどうか、および rate control モード (CRF/CQP/CBR/VBR) との組み合わせ制約

判定基準:

- **部分サポート** とするには、上記の調査対象のいずれかで QP を動的に変更できる経路が確認できること。ビットレートはピクチャー単位制御の対象ではないため確認事項 2 のスコープ外 (確認事項 1 でカバーする)
- 達成できる項目がキーフレーム強制 (`pic_type = EB_AV1_KEY_PICTURE`) のみの場合は、本クレートに既存実装があるため新規追加対象外とし **サポートなし** 扱い
- **部分サポート** と判定した場合、後続の別 issue では以下を含めて起票する: 達成可能と確認したフィールド / 必要な前提フラグ (`use_qp_file` 等) / rate control モードとの組み合わせ制約 / 公開する Rust API 名の案

## 解決方法

### 総合判定: サポートなし

`Encoder::reconfigure` を v4.1.0 で安全に提供する経路は存在しないため、本クレートには `Encoder::reconfigure` を追加しない。確認事項 2 で per-picture QP の経路自体は確認できたが、本 issue 根拠 (WebRTC / SFU 用途) は rate control モードが通常 CBR であり、`qp_on_the_fly` は CQP/CRF でしか実効しないため根拠と噛み合わず、後続 issue も起票しない。

### SVT-AV1 v4.1.0 の設計スタンス (判定の前提)

調査の結果、SVT-AV1 v4.1.0 はそもそもエンコード設定をランタイムで動的に再構成することを想定していない設計と判明した。

- 構造的パラメータ (解像度 / bitrate / rate control mode / プリセット / タイル数 / ref 数 / ビット深度等) は `svt_av1_enc_init` 時点で確定し、リソースプールとワーカースレッドがそのスナップショットを掴んで動き続ける。init 後に `svt_av1_enc_set_parameter` を呼び直しても `scs->static_config` のスカラーが書き換わるだけで、これらは作り直されない。
- 動的に効くのはピクチャー単位の限定的な制御のみ: `EbBufferHeaderType::pic_type = EB_AV1_KEY_PICTURE` によるキーフレーム強制と、`use_qp_file=true` かつ rate control が CQP/CRF のときの per-picture QP (`EbBufferHeaderType::qp`) のみ。
- WebRTC / SFU 用途で本来必要な「ビットレート / フレームレートの動的切替」を実現する on-the-fly reconfiguration は、機能要望 Issue #2040 として 2023-02-01 から 2 年以上 opened のまま v4.1.0 に届いていない。`svt_av1_enc_set_parameter` を多重呼び出ししたときのメモリリーク報告 Issue #2188 も同様に未修正。
- これは SVT-AV1 が AV1 リファレンス実装系の中でもバッチエンコード寄りに最適化されたエンコーダであり、姉妹クレート (aom-rs / libvpx-rs 等) のように WebRTC 文脈で `reconfigure` を提供する前提のライブラリではないことに起因する。

したがって本クレートに `Encoder::reconfigure` を追加しても、SVT-AV1 側がそもそも該当機能を提供していないため意味のある API にはならない。SVT-AV1 が将来 Issue #2040 を実装したタイミングで再検討する。

### v4.1.0 タグの基準情報

- タグ commit hash: `c04f951541ad600e0d9c10836f2ab7b9bc69816d`
- permalink ベース: `https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/v4.1.0/<path>`

### 確認事項 1 の調査結果: サポートなし

- `Source/API/EbSvtAv1Enc.h` の `svt_av1_enc_set_parameter` ドキュメントコメントは STEP 2 として単一フローを示すのみで、init 後再呼び出しを許容する文言はない。
  - permalink: <https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/c04f951541ad600e0d9c10836f2ab7b9bc69816d/Source/API/EbSvtAv1Enc.h#L1041-L1049>
- `Source/Lib/Globals/enc_handle.c` の `svt_av1_enc_set_parameter` 実装には `init_done` 相当の state チェック分岐が存在せず、init 後再呼び出し時に `prediction_structure_group_ptr` を旧オブジェクト解放なしに `EB_NO_THROW_NEW` で上書きするため、確定的にメモリリークが発生する。
  - permalink: <https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/c04f951541ad600e0d9c10836f2ab7b9bc69816d/Source/Lib/Globals/enc_handle.c#L4516-L4572>
  - `EB_NO_THROW_NEW` 定義: <https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/c04f951541ad600e0d9c10836f2ab7b9bc69816d/Source/Lib/Codec/object.h#L58-L68>
- `copy_api_from_app` と `set_param_based_on_input` は `scs->static_config.*` のスカラーを書き換えるのみ。`svt_av1_enc_init` が構築するリソースプール (`scs_pool_ptr` / `picture_parent_control_set_pool_ptr` / `me_pool_ptr` / `enc_dec_pool_ptr` / RC コンテキスト等) は再構築されず、ワーカースレッドが旧設定のスナップショットを保持し続ける。したがって解像度・プリセット・タイル数・ref 数・ビット深度などの構造的パラメータは反映されない。
- `Source/Lib/Globals/enc_settings.c` の `svt_av1_verify_settings` には init 前提のチェックは含まれないが、これは再呼び出しが安全であることを意味しない (verify 層が再呼び出しを考慮していないだけ)。
  - permalink: <https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/c04f951541ad600e0d9c10836f2ab7b9bc69816d/Source/Lib/Globals/enc_settings.c#L42>
- `Docs/` 配下の公式ドキュメント (`svt-av1_encoder_user_guide.md` / `Parameters.md` / `CommonQuestions.md` / `svt-av1-encoder-design.md`) には `svt_av1_enc_set_parameter` の API 名自体が登場せず、再呼び出しを許容する記述は存在しない。
- 公式 sample app (`Source/App/app_context.c:426-451`) は `svt_av1_enc_set_parameter` → `svt_av1_enc_init` を一度だけ呼ぶ単一経路で、init 後再呼び出しのコード経路は存在しない。
- 関連する GitLab Issue は v4.1.0 タグに未マージのまま:
  - リーク報告 #2188: <https://gitlab.com/AOMediaCodec/SVT-AV1/-/work_items/2188> (opened, 2024-06-04 作成 / 2024-06-10 最終更新)
  - on-the-fly reconfiguration 要望 #2040: <https://gitlab.com/AOMediaCodec/SVT-AV1/-/work_items/2040> (opened, 2023-02-01 作成 / 2025-01-21 最終更新)

### 確認事項 2 の調査結果: 経路は存在するが本 issue 根拠と不一致のため新規 issue は起票しない

- `EbBufferHeaderType::qp` が `pcs->picture_qp` に流れて qindex に反映される経路は確認できた。
  - 注入箇所: `Source/Lib/Codec/resource_coordination_process.c` の `use_qp_file` 分岐
    - permalink: <https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/v4.1.0/Source/Lib/Codec/resource_coordination_process.c>
  - qindex 反映箇所: `Source/Lib/Codec/rc_crf_cqp.c` の `svt_av1_rc_calc_qindex_crf_cqp`
    - permalink: <https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/v4.1.0/Source/Lib/Codec/rc_crf_cqp.c>
- 前提条件:
  - `EbSvtAv1EncConfiguration::use_qp_file = true` が必須 (`use_qp_file = false` のときは `picture_qp = scs->static_config.qp` で上書きされ、`EbBufferHeaderType::qp` は黙殺される)。
  - rate control モードは `SVT_AV1_RC_MODE_CQP_OR_CRF` (= 0) のときのみ実効。`Source/Lib/Codec/rc_vbr_cbr.c` には `qp_on_the_fly` / `picture_qp` の参照が一切なく、VBR/CBR では完全に黙殺される。
- `EbComponentType *` を取る init 後の OPTIONAL 関数は読み出し系 (`svt_av1_enc_stream_header` / `svt_av1_get_recon` / `svt_av1_enc_get_stream_info`) のみで、動的設定書き換え API は v4.1.0 に存在しない。
- 本 issue の根拠は WebRTC / SFU 用途のビットレート / フレームレート切替であり、これらの運用は通常 CBR で行われる。CQP/CRF 限定の per-picture QP API は本 issue の根拠を満たさないため、issue 本文に従えば「部分サポート」と判定して別 issue を起票するべきところ、根拠不一致のため後続 issue は起票しない (本クレートでは per-picture QP API を追加しない判断)。

### 後続アクション

- 本 issue を `issues/closed/` に移動する。
- 後続 issue の起票は行わない。
- CHANGES.md への追記は行わない (調査タスクのため機能変更なし)。
