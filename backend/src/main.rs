mod auth;
mod config;
mod database;
mod handlers;
mod models;
mod routes;

use anyhow::Result;
use axum::Router;
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::config::Config;
use crate::database::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Arc::new(Config::load()?);

    // Initialize database
    let pool = SqlitePool::connect(&config.database_url).await?;
    let db = Database::new(pool);
    db.migrate().await?;

    let state = AppState { db, config };

    // Build router
    let app = Router::new()
        .nest("/api", routes::create_routes())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;
    println!("Dr. Markdown server running on http://127.0.0.1:3001");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}