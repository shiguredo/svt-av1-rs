# カスタムログコールバックを追加する

- Created: 2026-08-06
- Completed: (未完了)
- Branch: feature/add-custom-log-callback
- Polished: {YYYY-MM-DD}

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
- 環境変数 `SVT_LOG`: コールバック登録済みなら SVT_LOG は読まれない (デフォルトロガーが使われないため)。コールバック未登録の利用者のために `std::env::set_var()` は残す
- レベルフィルタ: カスタムコールバックにはフィルタが無く全レベルが届く (SVT_AV1_LOG_ALL=-1 含む)。現状の「SVT_LOG=1 (error のみ)」挙動を維持するには Rust 側でフィルタを実装する。レベルは FATAL=0 / ERROR=1 / WARN=2 / INFO=3 / DEBUG=4

## 設計方針

- `svt_av1_set_log_callback` は「すべてのインスタンスに影響するグローバル設定」であり、プロセスに 1 回だけ登録できる API として設計する
- 転送先は `log` クレート (既存依存) とする。コールバック内で `log::log!` 相当のマクロを呼ぶ
- レベルフィルタは Rust 側で実装し、現状の error のみという挙動をデフォルトとして維持する
- コールバックの登録 API は「最初の `Encoder::new()` より前に呼ぶこと」を doc に明記する
- `SVT_LOG` を設定する `std::env::set_var()` はコールバック未登録の利用者のために残す

## 完了条件

- カスタムログコールバックを登録できる公開 API が追加されること
- 登録したコールバックにエンコーダのログが転送されること (`va_list` が `vsnprintf` で文字列化され、レベルフィルタを通って `log` クレートへ送出されること)
- コールバック未登録時は従来どおり `SVT_LOG` 環境変数で制御されること
- 既存テストスイートがパスすること
- `CHANGES.md` の develop セクションに [ADD] として追記すること

## 解決方法

- `src/lib.rs` にログコールバックの登録 API と `vsnprintf` による文字列化を実装する
- `cfg(target_os)` で macOS / Linux の `va_list` 型の差分を吸収する
- ログ転送のテストを追加する (コールバック登録後にエンコードし、ログが転送されることを確認する)
