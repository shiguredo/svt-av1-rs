# EncodedFrame::is_keyframe() の判定対象に Forward Key を追加する

- Created: 2026-08-02
- Completed: 2026-08-06
- Branch: feature/add-is-keyframe-forward-key
- Polished: 2026-08-06
- Reporter: @voluntas

## 目的

SVT-AV1 が将来 `EB_AV1_FW_KEY_PICTURE` (Forward Key) を出力するようになった場合に備え、`EncodedFrame::is_keyframe()` が Forward Key フレームをキーフレームとして検出できるようにする。

## 現状

`EncodedFrame::is_keyframe()` は `EB_AV1_KEY_PICTURE` と `EB_AV1_INTRA_ONLY_PICTURE` のみを判定している。

SVT-AV1 v4.2.0 では、出力パケットの pic_type は `packetization_process.c` の 4 分岐 (KEY / INTRA_ONLY / INTER / NON_REF) でのみ設定され、`EB_AV1_FW_KEY_PICTURE` は enum 定義 (EbSvtAv1.h) に存在するもののエンコード出力では一切使用されない。FwdkfRefresh 構成のキーフレームも `EB_AV1_KEY_PICTURE` または `EB_AV1_INTRA_ONLY_PICTURE` として出力される。

一方、`EncodedFrame::pic_type()` は `EB_AV1_FW_KEY_PICTURE` を `PictureType::ForwardKey` に変換しており (src/lib.rs)、「`ForwardKey` を返すのに `is_keyframe()` が false を返す」という API の理論上の不整合が存在する (v4.2.0 では `EB_AV1_FW_KEY_PICTURE` が出力されないため、実際には観測されない)。

## 設計方針

`is_keyframe()` の契約は「SVT-AV1 のピクチャタイプ意味論においてキーフレームに相当するかどうか」と定義し、`PictureType::ForwardKey` はキーフレームとして扱う。v4.2.0 では FwdkfRefresh のキーフレーム (CRA) は `EB_AV1_KEY_PICTURE` または `EB_AV1_INTRA_ONLY_PICTURE` として出力され、`EB_AV1_FW_KEY_PICTURE` は出力に現れないため挙動は変わらず、将来 SVT-AV1 が Forward Key を出力するようになった場合の防御的変更として位置づける。

## 完了条件

- `is_keyframe()` の判定に `EB_AV1_FW_KEY_PICTURE` が含まれること
- FwdkfRefresh 構成でエンコードしたとき、`intra_period_length + 1` (keyint) の周期の位置にキーフレーム (CRA) が出力され、そのすべてで `is_keyframe() == true` を返すこと (テストではキーフレームの出力位置が周期と一致することも確認する)
- 既存の閉じた GOP 構成の挙動が変わらないこと (既存の一致対象を保持する変更のため、既存テストスイートのパスとコード確認で担保する)
- `CHANGES.md` の develop セクションに [ADD] として追記すること

## 解決方法

- `src/lib.rs` の `EncodedFrame::is_keyframe()` の一致対象に `EB_AV1_FW_KEY_PICTURE` を追加した
- `is_keyframe()` の doc に判定対象 (KEY / INTRA_ONLY / Forward Key) と、SVT-AV1 のピクチャタイプに基づく判定であることを明記した (v4.2.0 では Forward Key は出力されない旨も追記)
- テストを `src/lib.rs` 内の `#[cfg(test)]` モジュールに追加した
  - `is_keyframe_fwdkf_refresh`: FwdkfRefresh 構成 (CqpOrCrf、intra_period_length=31 → keyint=32) で 64 フレームをエンコードし、pts=0 / pts=32 にキーフレームが出力されて `is_keyframe() == true` になること、周期キーフレーム (CRA) が IntraOnly として出力されること、インターフレームで `is_keyframe() == false` になることを確認する
  - `is_keyframe_key_picture`: 閉じた GOP 構成 (KfRefresh) の先頭フレームが KEY_PICTURE として出力され、`is_keyframe() == true` になることを確認する
  - `is_keyframe_forward_key`: `EB_AV1_FW_KEY_PICTURE` を直接構築して `is_keyframe() == true` になることを確認する (v4.2.0 では出力されないため直接構築で検証)
- `CHANGES.md` の develop セクションに [ADD] として追記した
