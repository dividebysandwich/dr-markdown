mod auth;
mod config;
mod database;
mod handlers;
mod models;
mod routes;
mod llm;

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

const APP_BASE: &str = match option_env!("LEPTOS_APP_BASE_PATH") {
    Some(path) => path,
    None => "",
};

const SERVER_ADDR: &str = match option_env!("SERVER_ADDR") {
    Some(addr) => addr,
    None => "127.0.0.1",
};

const OLLAMA_ADDR: &str = match option_env!("OLLAMA_ADDR") {
    Some(addr) => addr,
    None => "http://localhost:11434",
};

const OLLAMA_MODEL: &str = match option_env!("OLLAMA_MODEL") {
    Some(model) => model,
    None => "llama3.2",
};


fn get_server_port() -> u16 {
    match option_env!("SERVER_PORT") {
        Some(port) => port.parse().unwrap_or(3001),
        None => 3001,
    }
}

pub const THEME_LIGHT: &str = "light";
pub const THEME_DARK: &str = "dark";

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
    let addr = format!("{}:{}", SERVER_ADDR, get_server_port());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Dr. Markdown server running on http://{}", addr);

    axum::serve(listener, app).await?;
    
    Ok(())
}   
