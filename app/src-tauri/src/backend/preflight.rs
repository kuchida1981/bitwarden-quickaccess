use std::io;

use thiserror::Error;
use tokio::process::Command;

/// `bw` CLI が見つからない、または `bw serve` に対応していない場合のエラー。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PreflightError {
    #[error("`{0}` コマンドが見つかりません。Bitwarden CLI (bw) をインストールし、PATH に追加してください。")]
    BwNotFound(String),
    #[error("`{0} serve --help` が失敗しました。この bw CLI のバージョンは `bw serve` に対応していません。")]
    ServeUnsupported(String),
    #[error("`{0}` の実行に失敗しました: {1}")]
    ExecutionFailed(String, String),
}

/// `bw` コマンドの存在確認と `serve` サブコマンド対応の確認を行う。
pub async fn check_bw_cli() -> Result<(), PreflightError> {
    check_bw_cli_with("bw").await
}

/// テスト用に `bw` 実行ファイルのパスを差し替え可能にしたバージョン。
pub async fn check_bw_cli_with(bw_path: &str) -> Result<(), PreflightError> {
    let output = Command::new(bw_path).args(["serve", "--help"]).output().await;

    match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(_) => Err(PreflightError::ServeUnsupported(bw_path.to_string())),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Err(PreflightError::BwNotFound(bw_path.to_string()))
        }
        Err(err) => Err(PreflightError::ExecutionFailed(bw_path.to_string(), err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn reports_bw_not_found_when_command_missing() {
        let result = check_bw_cli_with("/nonexistent/path/to/bw-does-not-exist").await;
        assert_eq!(
            result,
            Err(PreflightError::BwNotFound(
                "/nonexistent/path/to/bw-does-not-exist".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn reports_serve_unsupported_when_help_fails() {
        let dir = std::env::temp_dir().join(format!("bwqa-preflight-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let fake_bw = dir.join("bw");
        fs::write(&fake_bw, "#!/bin/sh\nexit 1\n").unwrap();
        let mut perms = fs::metadata(&fake_bw).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_bw, perms).unwrap();

        let result = check_bw_cli_with(fake_bw.to_str().unwrap()).await;

        assert_eq!(
            result,
            Err(PreflightError::ServeUnsupported(
                fake_bw.to_str().unwrap().to_string()
            ))
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn passes_when_serve_help_succeeds() {
        let dir = std::env::temp_dir().join(format!("bwqa-preflight-test-ok-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let fake_bw = dir.join("bw");
        fs::write(&fake_bw, "#!/bin/sh\nexit 0\n").unwrap();
        let mut perms = fs::metadata(&fake_bw).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_bw, perms).unwrap();

        let result = check_bw_cli_with(fake_bw.to_str().unwrap()).await;

        assert_eq!(result, Ok(()));

        fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn reports_execution_failed_when_permission_denied() {
        let dir = std::env::temp_dir().join(format!("bwqa-preflight-test-perm-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let fake_bw = dir.join("bw");
        fs::write(&fake_bw, "#!/bin/sh\nexit 0\n").unwrap();
        let mut perms = fs::metadata(&fake_bw).unwrap().permissions();
        perms.set_mode(0o644); // No execute permissions
        fs::set_permissions(&fake_bw, perms).unwrap();

        let result = check_bw_cli_with(fake_bw.to_str().unwrap()).await;

        match result {
            Err(PreflightError::ExecutionFailed(path, err_msg)) => {
                assert_eq!(path, fake_bw.to_str().unwrap());
                assert!(
                    err_msg.to_lowercase().contains("permission denied")
                        || err_msg.to_lowercase().contains("os error 13"),
                    "Expected permission denied, got: {}",
                    err_msg
                );
            }
            other => panic!("Expected ExecutionFailed, got: {:?}", other),
        }

        fs::remove_dir_all(&dir).ok();
    }
}
