# EncodedFrame::is_keyframe() の判定対象に Forward Key を追加する

- Created: 2026-08-02
- Completed: (未完了)
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

- `src/lib.rs` の `EncodedFrame::is_keyframe()` の一致対象に `EB_AV1_FW_KEY_PICTURE` を追加する
- `is_keyframe()` の doc に判定対象のピクチャタイプと契約 (ピクチャタイプ意味論に基づく) を明記する
- FwdkfRefresh 構成でキーフレーム判定が機能することを検証する回帰テストを追加する (`src/lib.rs` 内の `#[cfg(test)]` モジュール、既存の encode 系テストと同じ場所)
  - `EncoderConfig::intra_refresh_type` に `IntraRefreshType::FwdkfRefresh` を設定し、`intra_period_length` を明示する (デフォルトでは周期キーフレームが約 5 秒 GOP 相当と遠く、短いテストでは周期 CRA が発生しない)
  - `intra_period_length` より多いフレーム数 (例: 2 倍) をエンコードし、先頭フレーム以外の周期キーフレームが少なくとも 1 つ出力されること (キーフレーム数 >= 2) を確認する (フレーム数が不足すると先頭フレームだけでテストが黙ってパスする)。キーフレームの出力位置 (フレームインデックス) が `intra_period_length + 1` (keyint) の周期と一致することも確認する。なお FwdkfRefresh では keyint が mini-gop サイズの倍数になるよう `intra_period_length` を選ぶこと (FwdkfRefresh は hierarchical_levels が 4 に強制されるため mini-gop サイズは 16。例: `intra_period_length` に 31 を選ぶと keyint は 32)
  - レート制御モードは `RcMode::CqpOrCrf` を使う (`RcMode::Vbr` は SVT-AV1 がシングルパスで `intra_refresh_type` を `KF_REFRESH` に強制上書きするため (enc_handle.c。本クレートには pass 設定がなく常にシングルパス)、FwdkfRefresh の検証が黙って行われない)。CBR は src/lib.rs が pred_structure を低遅延に強制するため FwdkfRefresh と組み合わせられず、実際に試すとエラーとハングが発生する。`RcMode::CqpOrCrf` を使う場合は `target_bit_rate = 0` に設定すること (src/lib.rs の `validate_config` の制約)
  - v4.2.0 では Forward Key が出力されないため、`EB_AV1_KEY_PICTURE` / `EB_AV1_INTRA_ONLY_PICTURE` が `is_keyframe() == true` を返すことを確認する (実測では CqpOrCrf で INTRA_ONLY_PICTURE が出力される。KEY_PICTURE の確認は既存テストスイートで担保する)。このテストは `EB_AV1_FW_KEY_PICTURE` の追加自体を直接検証できないが、v4.2.0 では出力されないため不可避であり、FwdkfRefresh 構成の既存挙動の回帰担保として位置づける
  - インターフレームでは `is_keyframe() == false` になることも確認する
- 既存テストスイート (encode_cbr / encode_crf 等) がパスすることと、既存の一致対象 (KEY / INTRA_ONLY) を保持することで完了条件の回帰を検証する
- `CHANGES.md` の develop セクションに [ADD] として追記する
- なお `Encoder::next_frame()` のシグネチャは 0008 で `Result<Option<EncodedFrame>, Error>` に変更される予定のため、テスト実装時に調整する
