fn main() {
    tauri_build::build();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo");
    let repo_root = std::path::Path::new(&manifest_dir).join("../..");

    // HEAD やタグが変更された場合に build.rs を再実行し、バージョン情報を最新化する
    println!("cargo:rerun-if-changed={}", repo_root.join(".git/HEAD").display());
    println!("cargo:rerun-if-changed={}", repo_root.join(".git/refs/tags").display());

    // git describe を用いて、直近のタグからの相対位置を含む動的なバージョン文字列を導出する。
    // .git ディレクトリや git コマンドが存在しない環境では、Cargo.toml のバージョン値にフォールバックする。
    let version = std::process::Command::new("git")
        .args(["describe", "--tags", "--always"])
        .current_dir(&manifest_dir)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")));

    println!("cargo:rustc-env=BWQA_DISPLAY_VERSION={version}");
}
