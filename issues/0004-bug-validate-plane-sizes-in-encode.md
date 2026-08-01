# Encoder::encode() がプレーン長の不正を検出しない

- Created: 2026-08-02
- Completed: (未完了)
- Branch: feature/fix-validate-plane-sizes-in-encode
- Polished: (未実施)
- Reporter: @voluntas

## 目的

不正なプレーン長のフレームを検出し、破損ストリームの黙った出力を防ぐ。

## 現状

`Encoder::encode()` は 3 プレーンの**合計長のみ**を検証しており、個々のプレーンの長さが不正でも合計が一致すれば `Ok` を返す。

実機で確認済み: 320x240 I420 (y=76800, u=v=19200) に対し、y を 4 バイト短く、u/v を 2 バイトずつ長くした (合計は一致) フレームを渡すと `Ok(())` が返る。プレーン境界がずれたまま SVT-AV1 に送信され、エラーなしで破損ストリームが出力される。

## 設計方針

`Encoder::encode()` で各プレーンの長さを `Encoder::plane_sizes()` の結果と個別に比較する。`plane_sizes()` はオーバーフロー時に `None` を返すため、`None` の場合もエラーとする。

## 完了条件

- 合計は一致するが個別プレーン長が不正なフレームが `Err` を返すこと
- 正常なフレームが従来どおりエンコードできること

## 解決方法

- `src/lib.rs` の `Encoder::encode()` の検証を合計長から個別プレーン長に変更する
- `FrameData` の doc に「各プレーンの長さは `EncoderConfig` のプレーンサイズと一致させること」を明記する
- プレーン長不一致のテストを追加する (既存の `color_format_mismatch` はフォーマット不一致で先に弾かれるため、このパスは現在テストされていない)
