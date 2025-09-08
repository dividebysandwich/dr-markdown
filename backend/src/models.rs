use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Document {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDocumentRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDocumentRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Document> for DocumentResponse {
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            title: doc.title,
            content: doc.content,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DocumentSummary {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Document> for DocumentSummary {
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            title: doc.title,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}