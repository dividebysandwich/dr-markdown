use gloo_net::http::{RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

use crate::models::*;

use crate::app::APP_BASE;

pub const API_URL_OVERRIDE: Option<&str> = option_env!("API_URL");

pub fn api_url() -> String {
    match API_URL_OVERRIDE {
        Some(url) => url.to_string(),
        None => {
            if APP_BASE.is_empty() {
                "/api".to_string()
            } else {
                format!("{}/api", APP_BASE)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ApiClient {
    token: Option<String>,
}

#[derive(Serialize)]
pub struct ChatApiRequest {
    pub context: String,
    pub message: String,
}

async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T, ApiError> {
    if response.ok() {
        response.json().await.map_err(|e| ApiError {
            error: format!("JSON parse error: {}", e),
            status: 0,
        })
    } else {
        let status = response.status();
        let mut error: ApiError = response.json().await.unwrap_or(ApiError {
            error: format!("API error: {}", status),
            status: 0,
        });
        error.status = status;
        Err(error)
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self { token: None }
    }

    pub fn with_token(token: String) -> Self {
        Self {
            token: Some(token),
        }
    }

    fn build_request(&self, method: &str, path: &str) -> RequestBuilder {
        let url = format!("{}{}", api_url(), path);
        let mut req = RequestBuilder::new(&url).method(gloo_net::http::Method::from_bytes(method.as_bytes()).unwrap());

        if let Some(token) = &self.token {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }

        req.header("Content-Type", "application/json")
    }

    async fn send_request_builder<T: DeserializeOwned>(
        &self,
        req: RequestBuilder,
    ) -> Result<T, ApiError> {
        let response = req.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
            status: 0,
        })?;
        handle_response(response).await
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<AuthResponse, ApiError> {
        let request = self
            .build_request("POST", "/auth/login")
            .json(&LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .map_err(|e| ApiError {
                error: format!("Serialization error: {}", e),
                status: 0,
            })?;

        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
            status: 0,
        })?;

        handle_response(response).await
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<AuthResponse, ApiError> {
        let request = self
            .build_request("POST", "/auth/register")
            .json(&RegisterRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .map_err(|e| ApiError {
                error: format!("Serialization error: {}", e),
                status: 0,
            })?;
        
        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
            status: 0,
        })?;

        handle_response(response).await
    }

    // Update user settings (e.g., theme)
    pub async fn update_user_settings(&self, theme: &str) -> Result<User, ApiError> {
        let request = self
            .build_request("PUT", "/auth/profile")
            .json(&SettingsRequest {
                theme: theme.to_string(),
            })
            .map_err(|e| ApiError {
                error: format!("Serialization error: {}", e),
                status: 0,
            })?;

        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
            status: 0,
        })?;

        handle_response(response).await
    }

    #[allow(dead_code)]
    pub async fn get_profile(&self) -> Result<User, ApiError> {
        let req_builder = self.build_request("GET", "/auth/profile");
        self.send_request_builder(req_builder).await
    }

    pub async fn get_documents(&self) -> Result<Vec<DocumentSummary>, ApiError> {
        let req_builder = self.build_request("GET", "/documents");
        self.send_request_builder(req_builder).await
    }

    pub async fn get_document(&self, id: Uuid) -> Result<Document, ApiError> {
        let req_builder = self.build_request("GET", &format!("/documents/{}", id));
        self.send_request_builder(req_builder).await
    }

    pub async fn create_document(&self, title: &str, content: Option<&str>) -> Result<Document, ApiError> {
        let request = self
            .build_request("POST", "/documents")
            .json(&CreateDocumentRequest {
                title: title.to_string(),
                content: content.map(|s| s.to_string()),
            })
            .map_err(|e| ApiError {
                error: format!("Serialization error: {}", e),
                status: 0,
            })?;

        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
            status: 0,
        })?;
        
        handle_response(response).await
    }

    pub async fn update_document(
        &self,
        id: Uuid,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Document, ApiError> {
        let request = self
            .build_request("PUT", &format!("/documents/{}", id))
            .json(&UpdateDocumentRequest {
                title: title.map(|s| s.to_string()),
                content: content.map(|s| s.to_string()),
            })
            .map_err(|e| ApiError {
                error: format!("Serialization error: {}", e),
                status: 0,
            })?;
        
        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
            status: 0,
        })?;

        handle_response(response).await
    }

    pub async fn delete_document(&self, id: Uuid) -> Result<(), ApiError> {
        let req_builder = self.build_request("DELETE", &format!("/documents/{}", id));
        let response = req_builder.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
            status: 0,
        })?;

        if response.ok() {
            Ok(())
        } else {
            let status = response.status();
            let mut error: ApiError = response.json().await.unwrap_or(ApiError {
                error: format!("API error: {}", status),
                status: 0,
            });
            error.status = status;
            Err(error)
        }
    }

    pub async fn create_share_link(&self, id: Uuid) -> Result<Document, ApiError> {
        let req_builder = self.build_request("POST", &format!("/documents/{}/share", id));
        let response = req_builder.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
            status: 0,
        })?;
        handle_response(response).await
    }

    pub async fn remove_share_link(&self, id: Uuid) -> Result<Document, ApiError> {
        let req_builder = self.build_request("DELETE", &format!("/documents/{}/share", id));
        let response = req_builder.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
            status: 0,
        })?;
        handle_response(response).await
    }

    pub async fn get_shared_document(&self, token: &str) -> Result<SharedDocument, ApiError> {
        let req_builder = self.build_request("GET", &format!("/shared/{}", token));
        self.send_request_builder(req_builder).await
    }

    pub async fn ollama_chat_streaming(&self, body: &impl serde::Serialize) -> Result<gloo_net::http::Response, gloo_net::Error> {
        self.build_request("POST", "/llm")
            .json(body)?
            .send()
            .await
    }
}