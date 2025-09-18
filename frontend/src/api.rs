use gloo_net::http::{RequestBuilder, Response}; // Import Response
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;

use crate::models::*;

pub const API_URL: &str = match option_env!("API_URL") {
    Some(path) => path,
    None => "http://localhost:3001/api",
};

#[derive(Debug, PartialEq)]
pub struct ApiClient {
    token: Option<String>,
}

#[derive(Serialize)]
struct ChatApiRequest {
    context: String,
    message: String,
}

#[derive(Deserialize)]
struct ChatApiResponse {
    reply: String,
}

async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T, ApiError> {
    if response.ok() {
        response.json().await.map_err(|e| ApiError {
            error: format!("JSON parse error: {}", e),
        })
    } else {
        // Try to parse an ApiError from the body, otherwise provide a generic error.
        let error: ApiError = response.json().await.unwrap_or(ApiError {
            error: format!("API error: {}", response.status_text()),
        });
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
        let url = format!("{}{}", API_URL, path);
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
            })?;

        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
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
            })?;
        
        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
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
            })?;

        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
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
            })?;

        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
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
            })?;
        
        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
        })?;

        handle_response(response).await
    }

    pub async fn delete_document(&self, id: Uuid) -> Result<(), ApiError> {
        let req_builder = self.build_request("DELETE", &format!("/documents/{}", id));
        let response = req_builder.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
        })?;

        if response.ok() {
            Ok(())
        } else {
            let error: ApiError = response.json().await.unwrap_or(ApiError {
                error: format!("API error: {}", response.status_text()),
            });
            Err(error)
        }
    }

    pub async fn ollama_chat(&self, context: &str, message: &str) -> Result<String, ApiError> {
        let request_body = ChatApiRequest {
            context: context.to_string(),
            message: message.to_string(),
        };

        let request = self
            .build_request("POST", "/llm")
            .json(&request_body)
            .map_err(|e| ApiError {
                error: format!("Serialization error: {}", e),
            })?;

        let response = request.send().await.map_err(|e| ApiError {
            error: format!("Network error: {}", e),
        })?;

        // `handle_response` is generic, so we parse the specific JSON structure here.
        if response.ok() {
            let chat_response: ChatApiResponse = response.json().await.map_err(|e| ApiError {
                error: format!("Deserialization error: {}", e),
            })?;
            Ok(chat_response.reply)
        } else {
            // Get error string from returned json
            let reason = response.json::<ApiError>().await.unwrap_or(ApiError {
                error: "Unknown error".to_string(),
            });
            Err(ApiError {
                error: format!("AI Error: {}", reason.error),
            })
        }
    }
}