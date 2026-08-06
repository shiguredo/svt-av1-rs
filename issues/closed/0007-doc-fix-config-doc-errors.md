# EncoderConfig のドキュメント誤りを修正する

- Created: 2026-08-02
- Completed: 2026-08-06
- Branch: feature/update-config-doc-errors
- Polished: 2026-08-06
- Reporter: @voluntas

## 目的

`EncoderConfig` の doc コメントと README の誤った説明を SVT-AV1 v4.2.0 の一次資料と一致させる。

## 現状

SVT-AV1 v4.2.0 のヘッダ (EbSvtAv1Enc.h)・検証コード (enc_settings.c)・公式ドキュメント (Parameters.md) と照合した結果、以下の誤りがある。

| 対象 | 現在の記載 (src/lib.rs) | 正しい内容 (範囲) |
|---|---|---|
| `EncoderConfig::tile_columns` / `tile_rows` | タイル列数 (並列処理用) / タイル行数 (並列処理用) | SVT-AV1 は log2 値 (0=分割なし, 1=2 分割)。列は 0-4、行は 0-6、かつタイル総数 (1 << 列) × (1 << 行) が 128 以下であること (Annex A.3)。型が `Option<NonZeroUsize>` のため、0 (分割なし) は `None` で表現する |
| `EncoderConfig::screen_content_mode` | (0=無効, 1=検出, 2=強制, 3=拡張検出) | (0-3) 0=None, 1=Block Copy + Palette, 2=content adaptive, 3=content adaptive (anti-alias aware)。1 と 2 の値の意味が誤っている |
| `EncoderConfig::recode_loop` | (0=無効, 1=キーフレームのみ, 2=全フレーム) | (0-4) 0=無効, 1=KF+最大帯域超過, 2=KF/ARF/GF のみ, 3=全フレーム, 4=プリセット依存 |
| `EncoderConfig::cdef_level` | (-1=自動, 0=無効, 1-5=レベル) | (-1, 0-4) -1=自動。5 は SVT-AV1 がエラーを返す (Parameters.md の [0-1] は --enable-cdef の範囲であり、cdef_level の範囲は enc_settings.c の検証コードを正とする) |
| `EncoderConfig::sframe_mode` | (1=STRICT, 2=NEAREST) | (1-4) 1=STRICT, 2=NEAREST, 3=FLEXIBLE (ミニゴップ調整), 4=DEC_POSI (デコード順で位置調整) |

README の設定表にも同種の誤りが存在する (screen_content_mode は範囲 (0-3) のみの記載で誤りはないが、他の 5 フィールド (tile_columns / tile_rows / recode_loop / cdef_level / sframe_mode) は誤り)。

## 設計方針

SVT-AV1 v4.2.0 の一次資料に合わせて doc コメントと README を修正する。doc コメントには一次資料の語彙を使い、値の範囲もあわせて明記する。`validate_config` の検証ロジックは変更しない (doc と README の修正のみ)。

`tile_columns` / `tile_rows` は型が `Option<NonZeroUsize>` のため、SVT-AV1 の 0 (分割なし) は `None` で表現される。doc には「None は分割なし (SVT-AV1 の 0 相当)。Some の値は log2 (1=2 分割)」と明記する。

## 完了条件

- 修正後の記載が一次資料 (SVT-AV1 v4.2.0 タグの EbSvtAv1Enc.h / enc_settings.c / Parameters.md) と一致すること (cdef_level の範囲は enc_settings.c の検証コードを正とする)
- `README.md` の設定表も正しい内容に更新されていること (`src/lib.rs` の doc コメントと整合すること)
- `CHANGES.md` の develop セクションの misc に [UPDATE] として追記すること (エントリは `src/lib.rs` の doc コメント修正を主語とし、`README.md` の変更は含めない)

## 解決方法

- `src/lib.rs` の `EncoderConfig` の 6 フィールド (tile_columns / tile_rows / screen_content_mode / recode_loop / cdef_level / sframe_mode) の doc コメントを SVT-AV1 v4.2.0 の一次資料 (EbSvtAv1Enc.h / enc_settings.c / Parameters.md) と一致するように修正した
  - tile_columns / tile_rows: log2 値 (0=分割なし, 1=2 分割) と範囲 (列 0-4・行 0-6)・タイル総数 128 以下の制約 (Annex A.3) を明記した。`None` は分割なし (SVT-AV1 の 0 相当)
  - screen_content_mode: 各値の意味 (0=None, 1=Block Copy + Palette, 2=content adaptive, 3=content adaptive (anti-alias aware)) を Parameters.md の語彙で明記した
  - recode_loop: 範囲 0-4 と各値の意味 (0=無効, 1=KF+最大帯域超過, 2=KF/ARF/GF のみ, 3=全フレーム (ビットレート制約に基づく), 4=プリセット依存) を明記した
  - cdef_level: 範囲 (-1=自動, 0=無効, 1-4=レベル) と、-1 未満・5 以上は SVT-AV1 がエラーを返すことを明記した (enc_settings.c の検証コードを正とした)
  - sframe_mode: 範囲 1-4 と各値の意味 (1=STRICT, 2=NEAREST, 3=FLEXIBLE (ミニゴップ調整), 4=DEC_POSI (デコード順で位置調整)) を明記した
- `README.md` の設定表の同じ 5 フィールド (tile_columns / tile_rows / recode_loop / cdef_level / sframe_mode) の誤りを修正した (screen_content_mode は範囲 (0-3) のみの記載で誤りがなかったため対象外)
- `CHANGES.md` の develop セクションの misc に [UPDATE] として追記した
- コード・テストの変更はない (doc と README の修正のみ。既存テストが全て成功することを確認した)
