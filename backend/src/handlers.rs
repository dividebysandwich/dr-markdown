use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::{AuthService, AuthUser},
    models::{
        AuthResponse, CreateDocumentRequest, CreateUserRequest, DocumentResponse,
        DocumentSummary, LoginRequest, UpdateDocumentRequest, UserResponse, SettingsRequest
    },
    AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check if registration is allowed
    if !state.config.allow_registration {
        return Err(AppError::RegistrationDisabled);
    }

    request.validate()?;

    // Check if user already exists
    if let Some(_) = state.db.find_user_by_username(&request.username).await? {
        return Err(AppError::UserAlreadyExists);
    }

    // Hash password
    let password_hash = AuthService::hash_password(&request.password)?;

    // Create user
    let user = state
        .db
        .create_user(&request.username, &password_hash)
        .await?;

    // Generate token
    let token = AuthService::generate_token(user.id, &state.config.jwt_secret)?;

    let response = AuthResponse {
        token,
        user: user.into(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    request.validate()?;

    // Find user
    let user = state
        .db
        .find_user_by_username(&request.username)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    // Verify password
    if !AuthService::verify_password(&request.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    // Generate token
    let token = AuthService::generate_token(user.id, &state.config.jwt_secret)?;

    let response = AuthResponse {
        token,
        user: user.into(),
    };

    Ok(Json(response))
}

pub async fn get_profile(auth_user: AuthUser, State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let user = state
        .db
        .find_user_by_id(auth_user.user_id)
        .await?
        .ok_or(AppError::UserNotFound)?;

    let response: UserResponse = user.into();
    Ok(Json(response))
}

pub async fn update_user_settings(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<SettingsRequest>,
) -> Result<impl IntoResponse, AppError> {

    request.validate()?;

    let updated_user = state
        .db
        .update_user_theme(auth_user.user_id, request.theme.as_str())
        .await?
        .ok_or(AppError::UserNotFound)?;

    let response: UserResponse = updated_user.into();
    Ok(Json(response))
}


pub async fn create_document(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateDocumentRequest>,
) -> Result<impl IntoResponse, AppError> {
    request.validate()?;

    let content = request.content.unwrap_or_default();

    let document = state
        .db
        .create_document(auth_user.user_id, &request.title, &content)
        .await?;

    let response: DocumentResponse = document.into();
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_documents(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let documents = state.db.find_documents_by_user(auth_user.user_id).await?;

    let summaries: Vec<DocumentSummary> = documents.into_iter().map(Into::into).collect();
    Ok(Json(summaries))
}

pub async fn get_document(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(document_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let document = state
        .db
        .find_document_by_id(document_id, auth_user.user_id)
        .await?
        .ok_or(AppError::DocumentNotFound)?;

    let response: DocumentResponse = document.into();
    Ok(Json(response))
}

pub async fn update_document(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(document_id): Path<Uuid>,
    Json(request): Json<UpdateDocumentRequest>,
) -> Result<impl IntoResponse, AppError> {
    request.validate()?;

    let document = state
        .db
        .update_document(
            document_id,
            auth_user.user_id,
            request.title.as_deref(),
            request.content.as_deref(),
        )
        .await?
        .ok_or(AppError::DocumentNotFound)?;

    let response: DocumentResponse = document.into();
    Ok(Json(response))
}

pub async fn delete_document(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(document_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = state
        .db
        .delete_document(document_id, auth_user.user_id)
        .await?;

    if !deleted {
        return Err(AppError::DocumentNotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Authentication error: {0}")]
    Auth(#[from] anyhow::Error),
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("Document not found")]
    Llm,
    #[error("Error communicating with LLM:")]
    DocumentNotFound,
    #[error("Registration is disabled")]
    RegistrationDisabled,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Auth(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UserAlreadyExists => StatusCode::CONFLICT,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::UserNotFound => StatusCode::NOT_FOUND,
            AppError::DocumentNotFound => StatusCode::NOT_FOUND,
            AppError::Llm => StatusCode::BAD_GATEWAY,
            AppError::RegistrationDisabled => StatusCode::FORBIDDEN,
        };

        let body = Json(serde_json::json!({
            "error": self.to_string()
        }));

        (status, body).into_response()
    }
}