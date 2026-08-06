# カスタムログコールバック対応の追加可否を調査する

- Created: 2026-03-19
- Completed: (未完了)
- Branch: feature/add-custom-log-callback
- Polished: 2026-08-06
- Model: Opus 4.6

## 概要

SVT-AV1 v4.0.0 で追加されたカスタムログコールバック API (`svt_av1_set_log_callback`) に対応できるかを調査する。本クレートが追従するバージョンは `Cargo.toml` の `[package.metadata.external-dependencies]` で固定されており、現行の v4.2.0 では `svt_av1_set_log_callback` / `SvtAv1LogCallback` が bindings に生成済みである。

## 判定結果の定義

本文中はこの 3 語のいずれかのみを使う。「判定 = 方針」と扱い、確定した時点で対応する処理フローへ移る。

- **対応可能**: 確認事項がすべて一次情報で裏取りでき、安全に実装できる見込みが立った場合。実装 issue を新規起票し、本 issue を `issues/closed/` へ移す
- **対応不可**: 安全な実装手段が一次情報で確認できない場合。本 issue を `issues/closed/` へ移す
- **保留**: 調査を尽くしても一次情報で結論できない場合に限る。本 issue を `issues/pending/` へ移し、pending にした理由を追記する

## 根拠

現在は SVT-AV1 のログ出力を環境変数 `SVT_LOG` で制御している。`src/lib.rs` の `Encoder::with_log_level()` が `std::env::set_var()` で毎回 `SVT_LOG` を設定しており、これはプロセスグローバルな設定である。カスタムログコールバックを使えば SVT-AV1 のログを Rust の `log` クレートに転送でき、アプリケーションのロガーに統合できる。

ただし `svt_av1_set_log_callback` も「すべてのインスタンスに影響するグローバル設定」であり、プロセスグローバル性は変わらないため、本 issue の目的は「ログの出力先を stderr から Rust 側のロガーに変更できるようにする」ことに限定する。

## 確認事項

以下を一次情報で裏取りする。

### 共通の調査手順

- SVT-AV1 v4.2.0 タグの commit hash を SVT-AV1 GitLab (`https://gitlab.com/AOMediaCodec/SVT-AV1`) で取得し、調査記録に残す
- 引用するヘッダ / 実装ファイルは v4.2.0 タグの permalink で参照する
- 確認事項 1 で対応不可が確定した場合は以降の確認事項を実施せず、判定を確定する

### 確認事項 1: `va_list` の文字列化手段

`SvtAv1LogCallback` のシグネチャ (bindings の `SvtAv1LogCallback`) は `va_list` を引数に含む。Rust 側に `va_list` を消費する手段はないため、文字列化には C の `vsnprintf` への pass-through が必要になる。以下を確認する:

- bindings に `vsnprintf` が生成済みか (macOS / Linux の両方)
- コールバック内で受けた `va_list` を `vsnprintf` へそのまま渡せるか (`va_list` は opaque だが、`vsnprintf` へ渡すことは C の規約上正当)
- プラットフォーム差分 (macOS の `__builtin_va_list` は `*mut c_char`、Linux の GCC では配列型) が実装に与える影響

判定基準: 上記すべてが確認でき pass-through で文字列化できる場合は **対応可能** と判断する。いずれかが成立しない場合 (例: どちらかのプラットフォームの bindings に `vsnprintf` が生成されていない、`va_list` を `vsnprintf` へ渡せない、プラットフォーム差分への対処手段がない) は **対応不可** を確定する。この調査結果 (pass-through の可否とプラットフォーム差分の結論) は実装 issue の起票時に引き継ぐ。

### 確認事項 2: 呼び出し制約

`svt_av1_set_log_callback` は「すべてのインスタンスに影響するグローバル設定」「`svt_av1_enc_init_handle()` より前に呼ぶ必要がある」「初回のログ出力前に登録した場合のみ有効 (ワンショット)」という制約を持つ。以下を確認する:

- `svt_av1_enc_init_handle()` の何度目の呼び出しまでに登録すれば有効か
- コールバックは SVT-AV1 の内部スレッドから呼ばれ得る。複数スレッドからの同時呼び出しの可能性、コールバックからロガーへ送出する際の並行安全性、context ポインタのライフタイムを確認する
- 登録位置 (`Encoder::new()` 内か、それとも別の入口か) は調査結果を踏まえて実装 issue で決定する

### 確認事項 3: 環境変数 `SVT_LOG` との関係

コールバックを登録すると「すべてのログメッセージはデフォルトの stderr / ファイル出力の代わりにコールバックへ配送される」。このとき環境変数 `SVT_LOG` の設定が不要になるか、コールバック未登録の利用者のために `std::env::set_var()` を残すかを確認する。コールバック未登録時に `set_var` を削除すると、デフォルトログレベルが現状の error から SVT-AV1 のデフォルト (info) に変わる点に注意する。転送先を `log` クレートにするか `tracing` にするかは実装 issue の判断事項とする。

### 確認事項 4: レベルフィルタリング

コールバックにはレベルフィルタが無く全レベルのメッセージが届く。現状の「`SVT_LOG=1` (fatal と error) のみ」という挙動を維持するには、Rust 側でレベルフィルタを実装する必要がある。bindings の `SvtAv1LogLevel_*` 定数と `log` クレートの `Level` の対応を確認する。確認事項 2 の登録位置とあわせて、調査結果は実装 issue の起票時に引き継ぐ。

## 解決方法

調査結果 (確認事項ごとの判定と、一次情報の参照先) をこのセクションに追記してから、判定結果に応じて次の手順で closed / pending へ移す。

- closed: `git mv issues/0001-add-custom-log-callback.md issues/closed/0001-add-custom-log-callback.md`
- pending: pending にした理由を追記してから `git mv issues/0001-add-custom-log-callback.md issues/pending/0001-add-custom-log-callback.md`

対応可能と判定した場合は実装 issue を新規起票し、`Branch:` に記した `feature/add-custom-log-callback` をブランチ名の候補とする。
