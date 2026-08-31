use std::path::{Path, PathBuf};

/// Homebrew経由でインストールされた`bw`が置かれる典型的な場所。
/// Apple Silicon(`/opt/homebrew`)を優先し、Intel/Rosetta Homebrew(`/usr/local`)を次点とする。
/// (`bitwarden-cli`はhomebrew-coreの正式formulaでnodeに依存しており、Homebrew由来のnodeで
/// `npm install -g @bitwarden/cli` した場合も同じ場所に着地する)
const KNOWN_BW_PATHS: &[&str] = &["/opt/homebrew/bin/bw", "/usr/local/bin/bw"];

/// PATHが通っていない環境(Finder起動・ログイン項目起動等)でも`bw`を発見できるよう、
/// 設定ファイルによる明示指定 → 既知のインストール先 → プロセス継承PATHの順に解決する。
/// ログインシェルをspawnしてPATHを取得する方式(旧`fix_path_env`)は、シェル起動ファイル
/// への依存による脆さ・複雑さがあったため採用しない(詳細は
/// `openspec/changes/resolve-bw-cli-path/design.md` 参照)。
pub fn resolve_bw_path() -> String {
    resolve_bw_path_with(config_file_path().as_deref(), KNOWN_BW_PATHS)
}

/// `$XDG_CONFIG_HOME/bw-quickaccess/bw_path.txt`(未設定時は`~/.config/bw-quickaccess/bw_path.txt`)。
/// `HOME`すら取得できない場合は`None`を返し、設定ファイルは使わず既知パス以降にフォールスルーする。
fn config_file_path() -> Option<PathBuf> {
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(config_home.join("bw-quickaccess").join("bw_path.txt"))
}

/// テスト用に設定ファイルパス・既知パス候補を差し替え可能にしたバージョン。
/// 設定ファイルに書かれたパスが実行可能ファイルとして存在しない場合はエラーにせず、
/// 既知パス探索・PATHフォールバックへ落とす(壊れた設定ファイルで起動不能にしないため)。
fn resolve_bw_path_with(config_path: Option<&Path>, known_paths: &[&str]) -> String {
    if let Some(config_path) = config_path {
        if let Some(path) = read_config_override(config_path) {
            return path;
        }
    }

    for candidate in known_paths {
        if is_executable_file(Path::new(candidate)) {
            return candidate.to_string();
        }
    }

    "bw".to_string()
}

fn read_config_override(config_path: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(config_path).ok()?;
    let path = contents.trim();
    if path.is_empty() {
        return None;
    }
    is_executable_file(Path::new(path)).then(|| path.to_string())
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(metadata) => metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_executable(path: &Path) {
        fs::write(path, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bwqa-bw-locate-test-{name}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn falls_back_to_bare_bw_when_nothing_found() {
        let dir = test_dir("nothing-found");
        let config_path = dir.join("bw_path.txt"); // 存在しない

        let result = resolve_bw_path_with(Some(&config_path), &[]);

        assert_eq!(result, "bw");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn uses_config_file_path_when_valid() {
        let dir = test_dir("config-valid");
        let fake_bw = dir.join("bw");
        make_executable(&fake_bw);
        let config_path = dir.join("bw_path.txt");
        fs::write(&config_path, fake_bw.to_str().unwrap()).unwrap();

        let result = resolve_bw_path_with(Some(&config_path), &[]);

        assert_eq!(result, fake_bw.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn config_file_takes_priority_over_known_paths() {
        let dir = test_dir("config-priority");
        let config_bw = dir.join("config-bw");
        make_executable(&config_bw);
        let known_bw = dir.join("known-bw");
        make_executable(&known_bw);
        let config_path = dir.join("bw_path.txt");
        fs::write(&config_path, config_bw.to_str().unwrap()).unwrap();

        let result = resolve_bw_path_with(Some(&config_path), &[known_bw.to_str().unwrap()]);

        assert_eq!(result, config_bw.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn falls_through_to_known_paths_when_config_file_missing() {
        let dir = test_dir("config-missing");
        let known_bw = dir.join("known-bw");
        make_executable(&known_bw);
        let config_path = dir.join("does-not-exist.txt");

        let result = resolve_bw_path_with(Some(&config_path), &[known_bw.to_str().unwrap()]);

        assert_eq!(result, known_bw.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn falls_through_to_known_paths_when_config_path_invalid() {
        let dir = test_dir("config-invalid");
        let known_bw = dir.join("known-bw");
        make_executable(&known_bw);
        let config_path = dir.join("bw_path.txt");
        fs::write(&config_path, dir.join("nonexistent-bw").to_str().unwrap()).unwrap();

        let result = resolve_bw_path_with(Some(&config_path), &[known_bw.to_str().unwrap()]);

        assert_eq!(result, known_bw.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn falls_through_to_known_paths_when_config_file_empty() {
        let dir = test_dir("config-empty");
        let known_bw = dir.join("known-bw");
        make_executable(&known_bw);
        let config_path = dir.join("bw_path.txt");
        fs::write(&config_path, "  \n").unwrap();

        let result = resolve_bw_path_with(Some(&config_path), &[known_bw.to_str().unwrap()]);

        assert_eq!(result, known_bw.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn picks_first_matching_known_path_in_order() {
        let dir = test_dir("known-order");
        let first = dir.join("first-bw");
        let second = dir.join("second-bw");
        make_executable(&first);
        make_executable(&second);
        let config_path = dir.join("does-not-exist.txt");

        let result = resolve_bw_path_with(
            Some(&config_path),
            &[first.to_str().unwrap(), second.to_str().unwrap()],
        );

        assert_eq!(result, first.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn skips_known_path_that_does_not_exist() {
        let dir = test_dir("known-skip");
        let missing = dir.join("missing-bw");
        let existing = dir.join("existing-bw");
        make_executable(&existing);
        let config_path = dir.join("does-not-exist.txt");

        let result = resolve_bw_path_with(
            Some(&config_path),
            &[missing.to_str().unwrap(), existing.to_str().unwrap()],
        );

        assert_eq!(result, existing.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn works_without_config_path_at_all() {
        let dir = test_dir("no-config-path");
        let known_bw = dir.join("known-bw");
        make_executable(&known_bw);

        let result = resolve_bw_path_with(None, &[known_bw.to_str().unwrap()]);

        assert_eq!(result, known_bw.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
    }
}
