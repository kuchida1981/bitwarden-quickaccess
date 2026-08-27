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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusInfo {
    pub lock_status: LockStatus,
    pub user_email: Option<String>,
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

impl LoginDetail {
    pub fn icon_domain(&self) -> Option<String> {
        let uri = self.uris.iter().find_map(|u| u.uri.as_deref())?;
        let parse_target = if uri.contains("://") {
            uri.to_string()
        } else {
            format!("https://{uri}")
        };
        reqwest::Url::parse(&parse_target)
            .ok()
            .and_then(|u| u.host_str().map(str::to_string))
    }
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
    #[serde(rename = "userEmail")]
    user_email: Option<String>,
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
        self.get_envelope_with_query(path, &[]).await
    }

    async fn get_envelope_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        let mut req = self.http.get(format!("{}{}", self.base_url, path));
        if !query.is_empty() {
            req = req.query(query);
        }
        let response = req.send().await?;
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

    /// `/status` を叩き、ロック状態とアカウントメールアドレスを返す。
    pub async fn status(&self) -> Result<StatusInfo, ClientError> {
        let data: StatusData = self.get_envelope("/status").await?;
        let lock_status = match data.template.status.as_str() {
            "unlocked" => LockStatus::Unlocked,
            "unauthenticated" => LockStatus::Unauthenticated,
            _ => LockStatus::Locked,
        };
        Ok(StatusInfo {
            lock_status,
            user_email: data.template.user_email,
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
        let list_data: ListData<VaultItemSummary> = self
            .get_envelope_with_query("/list/object/items", &[("search", query)])
            .await?;
        Ok(list_data.data)
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
        assert_eq!(status.lock_status, LockStatus::Locked);
        assert_eq!(status.user_email, Some("user@example.com".to_string()));
    }

    #[tokio::test]
    async fn status_parses_unlocked() {
        let base_url = spawn_mock(
            r#"{"success":true,"data":{"object":"template","template":{"serverUrl":"https://example.com","lastSync":null,"userEmail":"user@example.com","userId":"u1","status":"unlocked"}}}"#,
        )
        .await;
        let client = BwServeClient::with_base_url(base_url);
        let status = client.status().await.unwrap();
        assert_eq!(status.lock_status, LockStatus::Unlocked);
        assert_eq!(status.user_email, Some("user@example.com".to_string()));
    }

    #[tokio::test]
    async fn status_parses_without_user_email() {
        let base_url = spawn_mock(
            r#"{"success":true,"data":{"object":"template","template":{"serverUrl":"https://example.com","lastSync":null,"status":"locked"}}}"#,
        )
        .await;
        let client = BwServeClient::with_base_url(base_url);
        let status = client.status().await.unwrap();
        assert_eq!(status.lock_status, LockStatus::Locked);
        assert_eq!(status.user_email, None);
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

    #[test]
    fn login_detail_icon_domain() {
        let detail = LoginDetail {
            username: None,
            password: None,
            totp: None,
            uris: vec![
                UriEntry {
                    uri: Some("https://www.amazon.co.jp/".to_string()),
                },
                UriEntry {
                    uri: Some("https://example.com".to_string()),
                },
            ],
        };
        assert_eq!(detail.icon_domain(), Some("www.amazon.co.jp".to_string()));

        let http_detail = LoginDetail {
            username: None,
            password: None,
            totp: None,
            uris: vec![UriEntry {
                uri: Some("http://sub.example.com/path/to/page".to_string()),
            }],
        };
        assert_eq!(http_detail.icon_domain(), Some("sub.example.com".to_string()));

        let empty_uri_detail = LoginDetail {
            username: None,
            password: None,
            totp: None,
            uris: vec![
                UriEntry { uri: None },
                UriEntry {
                    uri: Some("https://google.com".to_string()),
                },
            ],
        };
        assert_eq!(empty_uri_detail.icon_domain(), Some("google.com".to_string()));

        let no_uris_detail = LoginDetail {
            username: None,
            password: None,
            totp: None,
            uris: vec![],
        };
        assert_eq!(no_uris_detail.icon_domain(), None);

        let none_uris_detail = LoginDetail {
            username: None,
            password: None,
            totp: None,
            uris: vec![UriEntry { uri: None }],
        };
        assert_eq!(none_uris_detail.icon_domain(), None);

        let userinfo_detail = LoginDetail {
            username: None,
            password: None,
            totp: None,
            uris: vec![UriEntry {
                uri: Some("https://user:pass@vault.example.com/".to_string()),
            }],
        };
        assert_eq!(
            userinfo_detail.icon_domain(),
            Some("vault.example.com".to_string())
        );

        let port_detail = LoginDetail {
            username: None,
            password: None,
            totp: None,
            uris: vec![UriEntry {
                uri: Some("https://vault.example.com:8443/".to_string()),
            }],
        };
        assert_eq!(
            port_detail.icon_domain(),
            Some("vault.example.com".to_string())
        );
    }
}
