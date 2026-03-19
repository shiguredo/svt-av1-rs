
# カスタムログコールバックに対応する

Created: 2026-03-19
Model: Opus 4.6

## 概要

SVT-AV1 v4.0.0 で追加されたカスタムログコールバック API (`svt_av1_set_log_callback`) に対応する。

## 根拠

現在は SVT-AV1 のログ出力を環境変数 `SVT_LOG` で制御しているが、これはプロセスグローバルで複数エンコーダーインスタンスで共有される。カスタムログコールバックを使うことで、Rust の `log` クレートやアプリケーション固有のロガーに SVT-AV1 のログを統合できる。

## 対応内容

- `svt_av1_set_log_callback` を呼び出して SVT-AV1 のログを Rust の `log` クレートに転送する
- 環境変数 `SVT_LOG` の設定を不要にする

## pending の理由

`SvtAv1LogCallback` のシグネチャが `va_list` を含むため、Rust から安全に扱うには FFI の複雑な処理が必要。`va_list` の取り扱いはプラットフォーム依存であり、安定した実装には追加の調査が必要。
