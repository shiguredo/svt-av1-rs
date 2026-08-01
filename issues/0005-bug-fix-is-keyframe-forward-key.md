# EncodedFrame::is_keyframe() が Forward Key フレームを検出しない

- Created: 2026-08-02
- Completed: (未完了)
- Branch: feature/fix-is-keyframe-forward-key
- Polished: (未実施)
- Reporter: @voluntas

## 目的

FwdkfRefresh (open GOP) 構成で Forward Key フレームがキーフレームとして検出されるようにする。

## 現状

`EncodedFrame::is_keyframe()` は `EB_AV1_KEY_PICTURE` と `EB_AV1_INTRA_ONLY_PICTURE` のみを判定している。`EB_AV1_FW_KEY_PICTURE` (Forward Key) を含まないため、`EncoderConfig::intra_refresh_type` に `IntraRefreshType::FwdkfRefresh` を指定した場合、Forward Key フレームが `false` を返す。

muxer 等がキーフレーム検出に `is_keyframe()` を使うと、open GOP 構成でキーフレームが検出されなくなる。

## 設計方針

`is_keyframe()` の判定に `EB_AV1_FW_KEY_PICTURE` を追加する。

## 完了条件

- FwdkfRefresh 構成で Forward Key フレームが `is_keyframe() == true` を返すこと
- 既存の閉じた GOP 構成の挙動が変わらないこと

## 解決方法

- `src/lib.rs` の `EncodedFrame::is_keyframe()` の一致対象に `EB_AV1_FW_KEY_PICTURE` を追加する
- FwdkfRefresh 構成でキーフレーム判定を検証するテストを追加する
