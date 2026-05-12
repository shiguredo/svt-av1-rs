# Encoder::reconfigure の追加可否を調査する

Created: 2026-05-12
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
