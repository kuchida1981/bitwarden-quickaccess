use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

/// `bw serve` のHTTP APIに対する薄いクライアントラッパー。
pub struct BwServeClient {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("bw serve へのHTTPリクエストに失敗しました: {0}")]
    Request(#[from] reqwest::Error),
    #[error("bw serve のレスポンスの解析に失敗しました: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("bw serve がエラーを返しました: {0}")]
    Api(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockStatus {
    Locked,
    Unlocked,
    Unauthenticated,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UriEntry {
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginDetail {
    pub username: Option<String>,
    pub password: Option<String>,
    pub totp: Option<String>,
    #[serde(default)]
    pub uris: Vec<UriEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VaultItemSummary {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub login: Option<LoginDetail>,
}

pub type VaultItemDetail = VaultItemSummary;

#[derive(Deserialize)]
struct ApiEnvelope<T> {
    success: bool,
    data: Option<T>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct StatusData {
    template: StatusTemplate,
}

#[derive(Deserialize)]
struct StatusTemplate {
    status: String,
}

#[derive(Deserialize)]
struct ListData<T> {
    data: Vec<T>,
}

#[derive(Deserialize)]
struct TotpData {
    data: String,
}

#[derive(Serialize)]
struct UnlockBody<'a> {
    password: &'a str,
}

impl BwServeClient {
    pub fn new(port: u16) -> Self {
        Self::with_base_url(format!("http://localhost:{port}"))
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get_envelope<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        let response = self.http.get(format!("{}{}", self.base_url, path)).send().await?;
        let text = response.text().await?;
        let envelope: ApiEnvelope<T> = serde_json::from_str(&text)?;
        if envelope.success {
            envelope
                .data
                .ok_or_else(|| ClientError::Api("成功レスポンスに data がありません".to_string()))
        } else {
            Err(ClientError::Api(
                envelope.message.unwrap_or_else(|| "unknown error".to_string()),
            ))
        }
    }

    async fn post_and_check_success<B: Serialize>(&self, path: &str, body: Option<&B>) -> Result<(), ClientError> {
        let mut req = self.http.post(format!("{}{}", self.base_url, path));
        if let Some(body) = body {
            req = req.json(body);
        }
        let response = req.send().await?;
        let text = response.text().await?;
        let envelope: ApiEnvelope<serde_json::Value> = serde_json::from_str(&text)?;
        if envelope.success {
            Ok(())
        } else {
            Err(ClientError::Api(
                envelope.message.unwrap_or_else(|| "unknown error".to_string()),
            ))
        }
    }

    /// `/status` を叩き、ロック状態を返す。
    pub async fn status(&self) -> Result<LockStatus, ClientError> {
        let data: StatusData = self.get_envelope("/status").await?;
        Ok(match data.template.status.as_str() {
            "unlocked" => LockStatus::Unlocked,
            "unauthenticated" => LockStatus::Unauthenticated,
            _ => LockStatus::Locked,
        })
    }

    /// `/unlock` にマスターパスワードを送り、成否を返す。
    pub async fn unlock(&self, password: &str) -> Result<(), ClientError> {
        self.post_and_check_success("/unlock", Some(&UnlockBody { password })).await
    }

    /// `/lock` を叩く。
    pub async fn lock(&self) -> Result<(), ClientError> {
        self.post_and_check_success::<serde_json::Value>("/lock", None).await
    }

    /// `/list/object/items?search=...` を叩き、アイテム一覧を返す。
    /// vaultがロックされている場合は `bw serve` のエラーメッセージがそのまま伝播する。
    pub async fn search_items(&self, query: &str) -> Result<Vec<VaultItemSummary>, ClientError> {
        let response = self
            .http
            .get(format!("{}/list/object/items", self.base_url))
            .query(&[("search", query)])
            .send()
            .await?;
        let text = response.text().await?;
        let envelope: ApiEnvelope<ListData<VaultItemSummary>> = serde_json::from_str(&text)?;
        if envelope.success {
            Ok(envelope.data.map(|d| d.data).unwrap_or_default())
        } else {
            Err(ClientError::Api(
                envelope.message.unwrap_or_else(|| "unknown error".to_string()),
            ))
        }
    }

    /// `/object/item/{id}` を叩き、アイテム詳細(username/password/totp/uris)を返す。
    pub async fn get_item(&self, id: &str) -> Result<VaultItemDetail, ClientError> {
        self.get_envelope(&format!("/object/item/{id}")).await
    }

    /// `/object/totp/{id}` を叩き、現在のTOTPコードを返す。
    pub async fn get_totp(&self, id: &str) -> Result<String, ClientError> {
        let data: TotpData = self.get_envelope(&format!("/object/totp/{id}")).await?;
        Ok(data.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// 1回だけリクエストを受け付け、固定のJSONボディを返すだけのモックHTTPサーバ。
    async fn spawn_mock(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 4096];
                let _ = socket.read(&mut buf).await;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
            }
        });

        format!("http://127.0.0.1:{port}")
    }

    #[tokio::test]
    async fn status_parses_locked() {
        let base_url = spawn_mock(
            r#"{"success":true,"data":{"object":"template","template":{"serverUrl":"https://example.com","lastSync":"2026-01-01T00:00:00.000Z","userEmail":"user@example.com","userId":"u1","status":"locked"}}}"#,
        )
        .await;
        let client = BwServeClient::with_base_url(base_url);
        let status = client.status().await.unwrap();
        assert_eq!(status, LockStatus::Locked);
    }

    #[tokio::test]
    async fn status_parses_unlocked() {
        let base_url = spawn_mock(
            r#"{"success":true,"data":{"object":"template","template":{"serverUrl":"https://example.com","lastSync":null,"userEmail":"user@example.com","userId":"u1","status":"unlocked"}}}"#,
        )
        .await;
        let client = BwServeClient::with_base_url(base_url);
        let status = client.status().await.unwrap();
        assert_eq!(status, LockStatus::Unlocked);
    }

    #[tokio::test]
    async fn search_items_locked_propagates_error() {
        let base_url = spawn_mock(r#"{"success":false,"message":"Vault is locked."}"#).await;
        let client = BwServeClient::with_base_url(base_url);
        let err = client.search_items("github").await.unwrap_err();
        match err {
            ClientError::Api(message) => assert_eq!(message, "Vault is locked."),
            other => panic!("expected ClientError::Api, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn search_items_unlocked_returns_items() {
        let base_url = spawn_mock(
            r#"{"success":true,"data":{"object":"list","data":[{"id":"1","name":"GitHub","login":{"username":"me","password":null,"totp":null,"uris":[{"uri":"https://github.com"}]}}]}}"#,
        )
        .await;
        let client = BwServeClient::with_base_url(base_url);
        let items = client.search_items("github").await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "GitHub");
        assert_eq!(items[0].login.as_ref().unwrap().username.as_deref(), Some("me"));
    }

    #[tokio::test]
    async fn get_totp_returns_code() {
        let base_url = spawn_mock(r#"{"success":true,"data":{"object":"totp","data":"123456"}}"#).await;
        let client = BwServeClient::with_base_url(base_url);
        let code = client.get_totp("1").await.unwrap();
        assert_eq!(code, "123456");
    }

    #[tokio::test]
    async fn unlock_success_returns_ok() {
        let base_url = spawn_mock(r#"{"success":true,"data":{"raw":"session-key"}}"#).await;
        let client = BwServeClient::with_base_url(base_url);
        client.unlock("master-password").await.unwrap();
    }

    #[tokio::test]
    async fn unlock_failure_propagates_message() {
        let base_url = spawn_mock(r#"{"success":false,"message":"Invalid master password."}"#).await;
        let client = BwServeClient::with_base_url(base_url);
        let err = client.unlock("wrong-password").await.unwrap_err();
        match err {
            ClientError::Api(message) => assert_eq!(message, "Invalid master password."),
            other => panic!("expected ClientError::Api, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn lock_success_returns_ok() {
        let base_url = spawn_mock(r#"{"success":true,"data":{"title":"Your vault is locked."}}"#).await;
        let client = BwServeClient::with_base_url(base_url);
        client.lock().await.unwrap();
    }
}
