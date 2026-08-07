# カスタムログコールバックを追加する

- Created: 2026-08-06
- Completed: (未完了)
- Branch: feature/add-custom-log-callback
- Polished: 2026-08-06

## 目的

SVT-AV1 のログ出力を stderr から Rust 側のロガー (`log` クレート) に転送できるようにし、アプリケーションのロガーに統合できるようにする。

## 現状

現在は SVT-AV1 のログ出力を環境変数 `SVT_LOG` で制御している。`src/lib.rs` の `Encoder::with_log_level()` が `std::env::set_var()` で毎回 `SVT_LOG` を設定しており、これはプロセスグローバルな設定である。

SVT-AV1 v4.2.0 には `svt_av1_set_log_callback` / `SvtAv1LogCallback` / `SvtAv1LogLevel` が存在し、bindings に生成済みである。ただしコールバックの引数に `va_list` が含まれており、Rust 側には `va_list` を消費する手段が無い。

## 調査結果の引き継ぎ

事前調査 (既存 issue) で確認済みの一次情報 (SVT-AV1 v4.2.0、tag commit `9292ec8e32bce26f781f277ec8739b53426c4300`):

- `va_list` の文字列化は bindings に生成済みの `vsnprintf` への pass-through で実現できる。コールバックの `args` と `vsnprintf` の第 4 引数は macOS / Linux の両方で型が一致する
- プラットフォーム差分: macOS では `va_list` は `*mut c_char`、Linux では `va_list` は `[__va_list_tag; 1]` (bindgen はコールバックの `args` を `*mut __va_list_tag` として生成)。コールバック関数定義の引数型は `cfg(target_os)` で分ける必要がある
- 登録タイミング: `svt_av1_set_log_callback` は「初回ログ出力より前」の登録のみ有効 (ワンショット)。`svt_av1_enc_init_handle()` が内部で多数のログを出力するため、実質「最初の `Encoder::new()` より前」に登録する必要がある
- スレッド: コールバックはエンコーダ内部スレッドから呼ばれ得る。ログ転送先はスレッドセーフであること。context ポインタはプロセス生存期間中有効である必要があり、Rust 側では `static` / `OnceLock` で保持する
- 環境変数 `SVT_LOG`: コールバック登録済みなら SVT_LOG は読まれない (デフォルトロガーが使われないため)
- レベルフィルタ: カスタムコールバックにはフィルタが無く全レベルが届く (SVT_AV1_LOG_ALL=-1 含む)。レベルは FATAL=0 / ERROR=1 / WARN=2 / INFO=3 / DEBUG=4。現状の `SVT_LOG=1` の挙動は FATAL と ERROR の両方を出力する (0001 の「error のみ」は誤り。svt_log.c のフィルタはレベルがしきい値より大きい場合のみスキップするため、SVT_LOG=1 では FATAL=0 と ERROR=1 が出力される)

## 設計方針

- `svt_av1_set_log_callback` は「すべてのインスタンスに影響するグローバル設定」であり、プロセスに 1 回だけ登録できる API として設計する
- 登録 API は `set_log_callback(level: Option<LogLevel>)` とし、最初の `Encoder::new()` より前に呼ぶことを doc に明記する。2 回目以降の呼び出しは `Err` を返す (登録状態は `static` / `OnceLock` で保持し、2 回目は拒否する)。`svt_av1_set_log_callback` は void を返し登録の成否を検出できないため、doc に「登録が初回ログ出力より遅れた場合は無効になる」旨も明記する
- レベルは bindings の `SvtAv1LogLevel` (c_int) と定数 (SVT_AV1_LOG_FATAL 等) が存在するが、`sys` モジュールは private であり、公開 API の引数に使うには re-export が必要になる (shiguredo-rust 規約では re-export は原則禁止)。独自の `LogLevel` enum (Fatal / Error / Warn / Info / Debug) を定義して公開 API に使い、bindings の定数との変換を内部で行う
- `set_log_callback()` のエラーは既存の `Error` 型で表現し、function 名で区別する (2 回目登録はコード `EB_ErrorBadParameter`、function 名に "already registered" を含める。レベルは Rust 側の enum で表現するため範囲外の値は存在しない)
- フィルタのしきい値は引数 `level` で指定する。`None` の場合は現状の `SVT_LOG=1` 相当 (FATAL / ERROR) をデフォルトとし、`Some(level)` の場合は「指定レベル以下を転送」とする (等値比較ではなく、FATAL=0 が漏れないようにする)。除外するのは ALL レベル (-1) のメッセージのみとする (`level == -1` で判定)。バナー (`svt_av1_print_version`) は SVT_INFO レベルであり、他の init 時 SVT_INFO とレベル・tag が同一で判別手段がないため、転送対象に含める (現状の `SVT_LOG=1` ではバナーも出力されないが、しきい値を INFO 以上にした場合は届く)
- 転送先は `log` クレート (既存依存) とする (shiguredo-rust 規約では tracing が指定されているが、既存コードが `log` を使用しており既存依存を優先する)。コールバック内で `log::log!` 相当のマクロを呼ぶ。レベル変換は FATAL / ERROR → `log::Level::Error`、WARN → Warn、INFO → Info、DEBUG → Debug とする
- コールバックの `tag` 引数は `log` クレートの target として渡す (NULL の場合は空文字列として扱う)
- `SVT_LOG` を設定する `std::env::set_var()` はコールバック未登録の利用者のために残す
- 登録 API は `Encoder::new()` の初期化と競合しないよう、既存の `GLOBAL_LOCK` で直列化する
- Windows (MSVC) の `va_list` は `char*` の typedef であり、macOS / Linux と bindgen の生成形式が異なる可能性がある。実装時に Windows 用の bindings を確認し、`cfg(target_os)` の分岐を追加する

## 完了条件

- カスタムログコールバックを登録できる公開 API (`set_log_callback(level: Option<LogLevel>)`) が追加されること。doc に「最初の `Encoder::new()` より前に呼ぶこと」「登録が初回ログ出力より遅れた場合は無効になること」が明記されていること
- 2 回目以降の `set_log_callback()` 呼び出しが `Err` を返すこと (エラーは既存の `Error` 型で function 名により区別されること)
- 登録したコールバックにエンコーダのログが転送されること (`va_list` が `vsnprintf` で文字列化され、レベルフィルタ (デフォルトは FATAL / ERROR) を通って `log` クレートへ送出されること。tag は target として渡されること)。ALL レベル (-1) のメッセージは転送されないこと
- コールバック未登録時は従来どおり `SVT_LOG` 環境変数で制御されること
- macOS / Linux / Windows のすべてでビルドが通り、既存テストスイートがパスすること (Windows の `va_list` は実装時に bindings を確認して対応する)
- ログ転送のテスト (別バイナリの独立プロセスで実行) と、2 回目登録の `Err` を検証するテストがパスすること
- `CHANGES.md` の develop セクションに [ADD] として追記すること

## 解決方法

- `src/lib.rs` にログコールバックの登録 API (`set_log_callback(level: Option<LogLevel>)`) と `vsnprintf` による文字列化を実装する
- 公開 API 用の `LogLevel` enum を定義し、bindings の `SvtAv1LogLevel` 定数との変換を内部で行う
- 登録状態は `static` / `OnceLock` で保持し、2 回目以降の登録は `Err` を返す。登録 API は `GLOBAL_LOCK` で直列化する
- `cfg(target_os)` で macOS / Linux / Windows の `va_list` 型の差分を吸収する
- ログ転送のテストを追加する
  - `src/lib.rs` の既存テストが同じプロセスで `Encoder::new()` を呼ぶとワンショット登録が間に合わなくなるため、ログ転送テストは既存の `tests/` ファイルとは別バイナリの独立プロセスで実行する (cargo test は同一バイナリ内のテスト関数を並列実行するため、テスト関数は 1 つのみに限定する)
  - `log::set_logger()` でキャプチャ用ロガーを登録し、エンコード時の SVT-AV1 のログが指定レベルで転送されることを確認する。正常エンコードでは ERROR が出力されないため、しきい値を INFO に設定して init 時に必ず出力される SVT_INFO (バナー含む) の転送を確認する
  - 2 回目登録の `Err` は `Encoder::new()` を呼ばない単体テスト (`src/lib.rs` の `#[cfg(test)]` モジュール内で登録を 2 回呼ぶ) で検証する
