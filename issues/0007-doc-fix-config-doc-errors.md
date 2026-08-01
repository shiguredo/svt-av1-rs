# EncoderConfig のドキュメント誤りを修正する

- Created: 2026-08-02
- Completed: (未完了)
- Branch: feature/fix-config-doc-errors
- Polished: (未実施)
- Reporter: @voluntas

## 目的

`EncoderConfig` の doc コメントと README の誤った説明を SVT-AV1 v4.2.0 の一次資料と一致させる。

## 現状

SVT-AV1 v4.2.0 のヘッダ (EbSvtAv1Enc.h)・検証コード (enc_settings.c)・公式ドキュメント (Parameters.md) と照合した結果、以下の誤りがある。

| 対象 | 現在の記載 | 正しい内容 |
|---|---|---|
| `EncoderConfig::tile_columns` / `tile_rows` | タイル列数 / タイル行数 | SVT-AV1 は log2 値 (0=分割なし, 1=2 分割) |
| `EncoderConfig::screen_content_mode` | 1=検出, 2=強制 | 1=強制 (IntraBC+Palette), 2=コンテンツ適応的検出, 3=アンチエイリアス考慮検出 (1 と 2 が入れ替わっている) |
| `EncoderConfig::recode_loop` | 1=キーフレームのみ, 2=全フレーム | 1=KF+最大帯域超過, 2=KF/ARF/GF のみ, 3=全フレーム, 4=プリセット依存 |
| `EncoderConfig::cdef_level` | 1-5=レベル | 有効値は -1〜4 (5 は SVT-AV1 がエラーを返す) |
| `EncoderConfig::sframe_mode` | 1=STRICT, 2=NEAREST | 1=STRICT, 2=NEAREST, 3=FLEXIBLE, 4=DEC_POSI |

README の設定表にも同じ誤りが存在する。

## 設計方針

SVT-AV1 v4.2.0 の一次資料に合わせて doc コメントと README を修正する。値の範囲もあわせて明記する。

## 完了条件

- 上記 5 件の誤りが一次資料と一致すること
- `src/lib.rs` の doc コメントと `README.md` の設定表が整合すること

## 解決方法

- `src/lib.rs` の `EncoderConfig` の各フィールドの doc コメントを修正する
- `README.md` の設定表を修正する
