use std::{
    path::{Path, PathBuf},
    process::Command,
};

use shiguredo_cmake::Config;

// 依存ライブラリの名前
const LIB_NAME: &str = "SVT-AV1";
const LINK_NAME: &str = "SvtAv1Enc";

fn main() {
    // Cargo.toml か build.rs が更新されたら、依存ライブラリを再ビルドする
    println!("cargo::rerun-if-changed=Cargo.toml");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_SOURCE_BUILD");

    // 各種変数やビルドディレクトリのセットアップ
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("infallible"));
    let output_metadata_path = out_dir.join("metadata.rs");
    let output_bindings_path = out_dir.join("bindings.rs");

    // 各種メタデータを書き込む
    let (git_url, version) = get_git_url_and_version();
    std::fs::write(
        output_metadata_path,
        format!(
            concat!(
                "pub const BUILD_METADATA_REPOSITORY: &str={:?};\n",
                "pub const BUILD_METADATA_VERSION: &str={:?};\n",
            ),
            git_url, version
        ),
    )
    .expect("failed to write metadata file");

    if std::env::var("DOCS_RS").is_ok() {
        // Docs.rs 向けのビルドでは curl ができないので build.rs の処理はスキップして、
        // 代わりに、ドキュメント生成時に最低限必要な定義だけをダミーで出力している。
        //
        // See also: https://docs.rs/about/builds
        std::fs::write(
            output_bindings_path,
            concat!(
                "pub struct EbErrorType;",
                "pub struct EbBufferHeaderType;",
                "pub struct EbSvtIOFormat;",
                "pub struct EbComponentType;",
            ),
        )
        .expect("write file error");
        return;
    }

    let output_lib_dir = if should_use_prebuilt() {
        download_prebuilt(&out_dir, &output_bindings_path)
    } else {
        build_from_source(&out_dir, &output_bindings_path, &version)
    };

    println!(
        "cargo::rustc-link-search=native={}",
        output_lib_dir.display()
    );
    println!("cargo::rustc-link-lib=static={LINK_NAME}");
}

// source-build feature が有効でなければ prebuilt を使用する
fn should_use_prebuilt() -> bool {
    std::env::var("CARGO_FEATURE_SOURCE_BUILD").is_err()
}

// prebuilt バイナリをダウンロードして配置する
fn download_prebuilt(out_dir: &Path, output_bindings_path: &Path) -> PathBuf {
    let platform = get_target_platform();
    let version = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION is not set");
    let base_url = format!("https://github.com/shiguredo/svt-av1-rs/releases/download/{version}");
    let archive_name = format!("lib{LINK_NAME}-{platform}.tar.gz");
    let checksum_name = format!("{archive_name}.sha256");

    let archive_url = format!("{base_url}/{archive_name}");
    let checksum_url = format!("{base_url}/{checksum_name}");

    let prebuilt_dir = out_dir.join("prebuilt");
    let _ = std::fs::remove_dir_all(&prebuilt_dir);
    std::fs::create_dir_all(&prebuilt_dir).expect("failed to create prebuilt directory");

    let archive_path = prebuilt_dir.join(&archive_name);
    let checksum_path = prebuilt_dir.join(&checksum_name);

    // アーカイブをダウンロード
    println!("Downloading prebuilt {LIB_NAME} from {archive_url}");
    curl_download(&archive_url, &archive_path);

    // チェックサムをダウンロード
    curl_download(&checksum_url, &checksum_path);

    // SHA256 を検証
    let expected_hash = std::fs::read_to_string(&checksum_path)
        .expect("failed to read checksum file")
        .split_whitespace()
        .next()
        .expect("empty checksum file")
        .to_string();
    verify_sha256(&archive_path, &expected_hash);

    // 展開
    extract_tar_gz(&archive_path, &prebuilt_dir);

    // lib ディレクトリにコピー
    let lib_dir = out_dir.join("lib");
    let _ = std::fs::remove_dir_all(&lib_dir);
    std::fs::create_dir_all(&lib_dir).expect("failed to create lib directory");

    let lib_filename = get_lib_filename();
    std::fs::copy(
        prebuilt_dir.join(&lib_filename),
        lib_dir.join(&lib_filename),
    )
    .unwrap_or_else(|e| panic!("failed to copy {lib_filename}: {e}"));

    // bindings.rs をコピー
    std::fs::copy(prebuilt_dir.join("bindings.rs"), output_bindings_path)
        .expect("failed to copy bindings.rs");

    // ダウンロードしたファイルを削除
    let _ = std::fs::remove_dir_all(&prebuilt_dir);

    lib_dir
}

// ソースからビルドする
fn build_from_source(out_dir: &Path, output_bindings_path: &Path, version: &str) -> PathBuf {
    let out_build_dir = out_dir.join("build/");
    let _ = std::fs::remove_dir_all(&out_build_dir);
    std::fs::create_dir(&out_build_dir).expect("failed to create build directory");

    // git clone でソースを取得する
    let (git_url, _) = get_git_url_and_version();
    let src_dir = out_build_dir.join(LIB_NAME);
    git_clone(&git_url, version, &src_dir);

    let input_header_path = src_dir.join("Source/API/EbSvtAv1Enc.h");

    // shiguredo_cmake が管理する CMake バイナリを使用する
    shiguredo_cmake::set_cmake_env();

    // 依存ライブラリをビルドする
    let dst = Config::new(&src_dir)
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("SVT_AV1_LTO", "OFF")
        .profile("Release")
        .build();

    // バインディングを生成する
    bindgen::Builder::default()
        .header(input_header_path.to_str().expect("invalid header path"))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(output_bindings_path)
        .expect("failed to write bindings");

    dst.join("lib")
}

// git clone でソースを取得する
fn git_clone(url: &str, version: &str, dest: &Path) {
    println!("Cloning {LIB_NAME} {version} from {url}");

    let success = Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(version)
        .arg(url)
        .arg(dest)
        .status()
        .is_ok_and(|status| status.success());

    if !success {
        panic!("failed to clone {LIB_NAME} from {url}");
    }
}

// curl でファイルをダウンロードする
fn curl_download(url: &str, dest: &Path) {
    let success = Command::new("curl")
        .arg("-fsSL")
        .arg("--retry")
        .arg("3")
        .arg("-o")
        .arg(dest)
        .arg(url)
        .status()
        .is_ok_and(|status| status.success());

    if !success {
        panic!("failed to download from {url}");
    }
}

// tar.gz を展開する
fn extract_tar_gz(archive: &Path, dest: &Path) {
    let success = Command::new("tar")
        .arg("-xzf")
        .arg(archive)
        .arg("-C")
        .arg(dest)
        .status()
        .is_ok_and(|status| status.success());

    if !success {
        panic!("failed to extract {}", archive.display());
    }
}

// OS コマンドを使って SHA256 ハッシュを計算する
fn compute_sha256(file_path: &Path) -> String {
    let output = if cfg!(target_os = "macos") {
        Command::new("shasum")
            .arg("-a")
            .arg("256")
            .arg(file_path)
            .output()
            .expect("failed to execute shasum")
    } else if cfg!(target_os = "windows") {
        Command::new("certutil")
            .arg("-hashfile")
            .arg(file_path)
            .arg("SHA256")
            .output()
            .expect("failed to execute certutil")
    } else {
        Command::new("sha256sum")
            .arg(file_path)
            .output()
            .expect("failed to execute sha256sum")
    };

    if !output.status.success() {
        panic!("SHA256 command failed for {}", file_path.display());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if cfg!(target_os = "windows") {
        // certutil の出力形式: "SHA256 hash of <path>:\n<hash>\nCertUtil: ..."
        stdout
            .lines()
            .nth(1)
            .expect("unexpected certutil output")
            .replace(' ', "")
            .to_lowercase()
    } else {
        // shasum/sha256sum の出力形式: "<hash>  <filename>"
        stdout
            .split_whitespace()
            .next()
            .expect("unexpected sha256 output")
            .to_lowercase()
    }
}

// ファイルの SHA256 ハッシュを検証する
fn verify_sha256(file_path: &Path, expected_hash: &str) {
    println!("Verifying SHA256 hash for {}", file_path.display());

    let calculated_hash = compute_sha256(file_path);

    if calculated_hash.eq_ignore_ascii_case(expected_hash) {
        println!("=> SHA256 hash verified: {calculated_hash}");
    } else {
        panic!("SHA256 hash mismatch!\nExpected: {expected_hash}\nCalculated: {calculated_hash}");
    }
}

// ターゲット OS に応じたライブラリファイル名を返す
fn get_lib_filename() -> String {
    let os = std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    if os == "windows" {
        format!("{LINK_NAME}.lib")
    } else {
        format!("lib{LINK_NAME}.a")
    }
}

// ターゲットプラットフォームを判定する
fn get_target_platform() -> String {
    let os = std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");

    match (os.as_str(), arch.as_str()) {
        ("linux", "x86_64") => {
            let version_id = get_ubuntu_version_id();
            format!("ubuntu-{version_id}_x86_64")
        }
        ("linux", "aarch64") => {
            let version_id = get_ubuntu_version_id();
            format!("ubuntu-{version_id}_arm64")
        }
        ("macos", "aarch64") => "macos_arm64".to_string(),
        ("windows", "x86_64") => "windows_x86_64".to_string(),
        _ => {
            panic!("unsupported platform: {os}-{arch}");
        }
    }
}

// Ubuntu のバージョン ID を取得する
fn get_ubuntu_version_id() -> String {
    let content =
        std::fs::read_to_string("/etc/os-release").expect("failed to read /etc/os-release");
    for line in content.lines() {
        if let Some(version) = line.strip_prefix("VERSION_ID=") {
            return version.trim_matches('"').to_string();
        }
    }
    panic!("VERSION_ID not found in /etc/os-release");
}

// Cargo.toml をパースしてメタデータテーブルを取得する
fn get_metadata() -> shiguredo_toml::Value {
    shiguredo_toml::Value::Table(
        shiguredo_toml::from_str(include_str!("Cargo.toml")).expect("failed to parse Cargo.toml"),
    )
}

// Cargo.toml から依存ライブラリの Git URL とバージョンタグを取得する
fn get_git_url_and_version() -> (String, String) {
    let cargo_toml = get_metadata();
    if let Some((Some(git_url), Some(version))) = cargo_toml
        .get("package")
        .and_then(|v| v.get("metadata"))
        .and_then(|v| v.get("external-dependencies"))
        .and_then(|v| v.get(LIB_NAME))
        .map(|v| {
            (
                v.get("url").and_then(|s| s.as_str()),
                v.get("version").and_then(|s| s.as_str()),
            )
        })
    {
        (git_url.to_string(), version.to_string())
    } else {
        panic!(
            "Cargo.toml does not contain a valid [package.metadata.external-dependencies.{LIB_NAME}] table"
        );
    }
}
