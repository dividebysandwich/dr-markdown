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
use tower_http::services::{ServeDir, ServeFile};

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
    
    // Resolve the frontend dist directory.
    // Check relative to the executable first, then relative to CWD.
    let frontend_dir = {
        let candidates = [
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .map(|p| p.join("../../frontend/dist")),
            Some(std::path::PathBuf::from("../frontend/dist")),
        ];
        candidates
            .into_iter()
            .flatten()
            .find(|p| p.join("index.html").exists())
    };

    let app = if let Some(frontend_dir) = frontend_dir {
        let frontend_dir = frontend_dir.canonicalize().unwrap_or(frontend_dir);
        println!("Serving frontend from: {:?}", frontend_dir);

        let index_file = frontend_dir.join("index.html");
        let serve_dir = ServeDir::new(&frontend_dir)
            .fallback(ServeFile::new(&index_file));

        Router::new()
            .nest("/api", routes::create_routes())
            .fallback_service(serve_dir)
            .layer(CorsLayer::permissive())
            .with_state(state)
    } else {
        println!("Note: Frontend dist directory not found, serving API only");
        println!("  CWD: {:?}", std::env::current_dir().ok());
        println!("  EXE: {:?}", std::env::current_exe().ok());
        Router::new()
            .nest("/api", routes::create_routes())
            .layer(CorsLayer::permissive())
            .with_state(state)
    };

    // Start server
    let addr = format!("{}:{}", SERVER_ADDR, get_server_port());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Dr. Markdown server running on http://{}", addr);

    axum::serve(listener, app).await?;
    
    Ok(())
}   
