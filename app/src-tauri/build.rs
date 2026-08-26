fn main() {
    tauri_build::build();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo");
    let repo_root = std::path::Path::new(&manifest_dir).join("../..");

    // HEADが変更された場合(新しいコミット等)に build.rs を再実行し、バージョン情報を
    // 最新化する。`.git/refs/tags` も監視対象に加える案があったが、存在しないパスを
    // 指定するとCargoが常に「変更あり」とみなし毎回再ビルドが走ってしまう(タグの無い
    // shallow clone等で顕在化する)上に、新規タグ追加だけでは実際には確実に再ビルドを
    // トリガーしないことも実機検証で判明したため、`.git/HEAD` のみを監視する。
    println!("cargo:rerun-if-changed={}", repo_root.join(".git/HEAD").display());

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
