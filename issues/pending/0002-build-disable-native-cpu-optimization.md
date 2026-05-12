### SVT-AV1 ビルド時に CPU ネイティブ最適化が有効になり、CI ビルドのバイナリが古い CPU で動かない可能性がある

Created: 2026-04-08
Model: Opus 4.6

## 背景

`build.rs` の `build_from_source` では SVT-AV1 を CMake 経由でビルドしているが、CPU 最適化に関するオプションを明示的に指定していない。

```rust
let dst = Config::new(&src_dir)
    .define("BUILD_SHARED_LIBS", "OFF")
    .define("SVT_AV1_LTO", "OFF")
    .profile("Release")
    .build();
```

SVT-AV1 の CMake には `NATIVE` (`-march=native` 相当) を制御するオプションがあり、バージョン・環境によってはデフォルトでビルドホストの CPU 機能を前提としたコードが生成される可能性がある。CI でビルドしたバイナリを配布する prebuilt 経路では、ビルドホストより古い CPU を持つ実行環境で `SIGILL` 等により動作しない恐れがある。

SVT-AV1 はもともと SIMD 命令のランタイムディスパッチ (`svt_aom_get_cpu_flags` 等) を備えているため、コンパイル時にホスト CPU を前提にする必要はない。

## 再現条件

- `source-build` feature でビルドする、もしくは prebuilt を生成する CI ジョブが新しい CPU (AVX-512 など) を持つマシン上で動作する場合
- 実行環境がビルドマシンより古い CPU を持つ場合

## 対応案

`build.rs` の CMake 設定で CPU ネイティブ最適化を明示的に無効化する。

```rust
let dst = Config::new(&src_dir)
    .define("BUILD_SHARED_LIBS", "OFF")
    .define("SVT_AV1_LTO", "OFF")
    .define("NATIVE", "OFF")
    .profile("Release")
    .build();
```

加えて、必要に応じて以下も検討する。

- `CMAKE_C_FLAGS` / `CMAKE_CXX_FLAGS` に `-march=native` 等が混入していないか確認
- prebuilt 生成 CI ワークフローで、想定する最低 CPU 世代を明文化

## 確認事項

- 採用している SVT-AV1 のバージョンにおける `NATIVE` オプションのデフォルト値
- prebuilt を生成している GitHub Actions ランナーの CPU と、配布先で想定する最低 CPU 世代の整合

## pending 理由

調査の結果、本 issue で懸念されている問題は現状の構成では実際には発生しない見込みのため、対応の必須性が低いと判断して pending に移す。

調査結果（2026-05-12 時点）:

- 採用している SVT-AV1 v4.1.0 の `CMakeLists.txt` では `option(NATIVE "Build for native performance (march=native)" OFF)` と定義されており、`NATIVE` オプションのデフォルトは `OFF`。`-march=native` は付与されない。
- 同 `CMakeLists.txt` では非 MSVC 環境で `-mno-avx` が明示的に付与されており、ベースラインを意図的に低く取る方針となっている。
- `ENABLE_AVX512` 等は AVX512 命令を含むコードを「ビルド」するか否かのオプションであり、SVT-AV1 はランタイムディスパッチ (`svt_aom_get_cpu_flags`) で実行時に CPU 機能を判定するため、AVX512 非対応 CPU でも動作する。
- `build.rs` の `Config::new` では `CMAKE_C_FLAGS` / `CMAKE_CXX_FLAGS` を渡しておらず、`release.yml` でも `CFLAGS` 等は設定していないため、`-march=native` が外部から混入する経路もない。

ただし以下の理由から将来的に再検討する価値はある:

- SVT-AV1 上流が将来 `NATIVE` のデフォルト値を変更する可能性
- 利用者が環境変数 `CFLAGS=-march=native` 等を持ち込んだ際の保険として、`build.rs` 側で `NATIVE=OFF` を明示しておく意義
- prebuilt を生成する GitHub Actions ランナーの CPU 世代と配布先の最低 CPU 世代の整合の明文化
