# Encoder::encode() がプレーン長の不正を検出しない

- Created: 2026-08-02
- Completed: 2026-08-06
- Branch: feature/fix-validate-plane-sizes-in-encode
- Polished: 2026-08-06
- Reporter: @voluntas

## 目的

不正なプレーン長のフレームを検出し、破損ストリームの黙った出力を防ぐ。

## 現状

`Encoder::encode()` は 3 プレーンの**合計長のみ**を検証しており、個々のプレーンの長さが不正でも合計が一致すれば `Ok` を返す。

実機で確認済み: 320x240 I420 (y=76800, u=v=19200) に対し、y を 4 バイト短く、u/v を 2 バイトずつ長くした (合計は一致) フレームを渡すと `Ok(())` が返る。プレーン境界がずれたまま SVT-AV1 に送信され、エラーなしで破損ストリームが出力される。

## 設計方針

`Encoder::encode()` で各プレーンの長さを `Encoder::plane_sizes()` の結果と個別に比較する。検証はフレームデータのコピーより前に行い、不正なフレームはコピーせずにエラーにする。

`plane_sizes()` はオーバーフロー時に `None` を返すが、`Encoder::validate_config()` が構築時に width / height / color_format の検証を済ませているため `encode()` 時点では発生しない想定。防御として `None` の場合もエラーとする。

## 完了条件

- 合計は一致するが個別プレーン長が不正なフレームが `Err` を返すこと (再現ケース: 320x240 I420 で y=76796, u=19202, v=19202 のフレーム)
- 長さエラーの内容から不正なプレーン (Y / U / V) を判別できること (エラー表示に不正プレーン名が含まれることをテストで確認する)
- 正常なフレームが従来どおりエンコードできること
- `CHANGES.md` の develop セクションに [FIX] として追記すること

## 解決方法

- `src/lib.rs` の `Encoder` 構造体に `height` フィールドを追加した
- `Encoder::encode()` 内で `Self::plane_sizes()` を再計算し、各プレーンの長さ (Y / U / V) を個別に比較するようにした。既存の合計長チェックはこの個別プレーン長チェックで置き換えた (個別一致なら合計も必ず一致するため)
- 長さエラーは不正なプレーンを特定できる function 名 (`shiguredo_svt_av1::Encoder::encode (Y / U / V plane size mismatch)`) にし、既存の「`shiguredo_svt_av1::Encoder::encode (color format mismatch)`」と同じ形式に合わせた
- `plane_sizes()` が `None` を返す場合 (通常は `validate_config` が弾くため発生しない想定) は防御としてエラーにした
- `FrameData` の doc と `Encoder::encode()` の doc に「各プレーンの長さは `EncoderConfig` の width / height / color_format から計算されるプレーンサイズと一致させること」を明記した (バイト単位・`ceil` 表記つき)
- テスト: `src/lib.rs` 内の `#[cfg(test)]` モジュールにプレーンサイズ不一致のテストを追加した
  - `plane_size_mismatch_y` / `_u` / `_v` (I420、合計長一致の再現ケース含む)
  - `plane_size_mismatch_10bit_y` / `_u` / `_v` (I42010)
  - `encode_after_plane_size_mismatch` (エラー後も正常フレームでエンコード継続できること)
  - `plane_sizes_odd_dimensions` (奇数寸法の `ceil` 計算の契約)
