use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

use crate::models::{Document, User};
use crate::THEME_LIGHT;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new Database, ensuring the parent directory for the SQLite file exists.
    pub async fn new_with_path(database_url: &str) -> Result<Self> {
        if let Some(path) = database_url.strip_prefix("sqlite:") {
            let path = path.trim_start_matches("//").trim_start_matches('/');
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent).ok();
            }
        }
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    // User operations
    pub async fn create_user(&self, username: &str, password_hash: &str) -> Result<User> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let theme = THEME_LIGHT; // default theme

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, password_hash, created_at, updated_at, theme)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING 
                id as "id: Uuid", 
                username, 
                password_hash, 
                created_at as "created_at: DateTime<Utc>", 
                updated_at as "updated_at: DateTime<Utc>",
                theme
            "#,
            id,
            username,
            password_hash,
            now,
            now,
            theme
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_user_theme(&self, user_id: Uuid, theme: &str) -> Result<Option<User>> {
        let now = Utc::now();

        let existing = self.find_user_by_id(user_id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET theme = ?, updated_at = ?
            WHERE id = ?
            RETURNING 
                id as "id: Uuid", 
                username, 
                password_hash, 
                created_at as "created_at: DateTime<Utc>", 
                updated_at as "updated_at: DateTime<Utc>" ,
                theme
            "#,
            theme,
            now,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(user))
    }

    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id as "id: Uuid", 
                username, 
                password_hash, 
                created_at as "created_at: DateTime<Utc>", 
                updated_at as "updated_at: DateTime<Utc>",
                theme
            FROM users WHERE username = ?
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User, 
            r#"
            SELECT 
                id as "id: Uuid", 
                username, 
                password_hash, 
                created_at as "created_at: DateTime<Utc>", 
                updated_at as "updated_at: DateTime<Utc>",
                theme
            FROM users WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    // Document operations
    pub async fn create_document(
        &self,
        user_id: Uuid,
        title: &str,
        content: &str,
    ) -> Result<Document> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let document = sqlx::query_as!(
            Document,
            r#"
            INSERT INTO documents (id, user_id, title, content, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING 
                id as "id: Uuid", 
                user_id as "user_id: Uuid", 
                title, 
                content, 
                created_at as "created_at: DateTime<Utc>", 
                updated_at as "updated_at: DateTime<Utc>"
            "#,
            id,
            user_id,
            title,
            content,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn find_document_by_id(
        &self,
        document_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Document>> {
        let document = sqlx::query_as!(
            Document,
            r#"
            SELECT 
                id as "id: Uuid", 
                user_id as "user_id: Uuid", 
                title, 
                content, 
                created_at as "created_at: DateTime<Utc>", 
                updated_at as "updated_at: DateTime<Utc>"
            FROM documents WHERE id = ? AND user_id = ?
            "#,
            document_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn find_documents_by_user(&self, user_id: Uuid) -> Result<Vec<Document>> {
        let documents = sqlx::query_as!(
            Document,
            r#"
            SELECT 
                id as "id: Uuid", 
                user_id as "user_id: Uuid", 
                title, 
                content, 
                created_at as "created_at: DateTime<Utc>", 
                updated_at as "updated_at: DateTime<Utc>"
            FROM documents WHERE user_id = ? ORDER BY updated_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(documents)
    }

    pub async fn update_document(
        &self,
        document_id: Uuid,
        user_id: Uuid,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Option<Document>> {
        let now = Utc::now();

        let existing = self.find_document_by_id(document_id, user_id).await?;
        if existing.is_none() {
            return Ok(None);
        }

        let existing = existing.unwrap();

        let new_title = title.unwrap_or(&existing.title);
        let new_content = content.unwrap_or(&existing.content);

        let document = sqlx::query_as!(
            Document,
            r#"
            UPDATE documents
            SET title = ?, content = ?, updated_at = ?
            WHERE id = ? AND user_id = ?
            RETURNING 
                id as "id: Uuid", 
                user_id as "user_id: Uuid", 
                title, 
                content, 
                created_at as "created_at: DateTime<Utc>", 
                updated_at as "updated_at: DateTime<Utc>"
            "#,
            new_title,
            new_content,
            now,
            document_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(document))
    }

    pub async fn delete_document(&self, document_id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM documents WHERE id = ? AND user_id = ?",
            document_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}