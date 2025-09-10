use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers, AppState};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/profile", get(handlers::get_profile))
        .route("/auth/profile", put(handlers::update_user_settings))
        .route("/documents", post(handlers::create_document))
        .route("/documents", get(handlers::get_documents))
        .route("/documents/{id}", get(handlers::get_document))
        .route("/documents/{id}", put(handlers::update_document))
        .route("/documents/{id}", delete(handlers::delete_document))
}