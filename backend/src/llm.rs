
use serde::{Deserialize, Serialize};
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    AppState,
    handlers::AppError,
    OLLAMA_ADDR,
    OLLAMA_MODEL,
};

// What the frontend will send to our backend
#[derive(Deserialize)]
pub struct ChatApiRequest {
    pub context: String,
    pub message: String,
}

// What we will send to Ollama's /api/generate endpoint
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

// The response we expect from Ollama
#[derive(Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    // ... other fields we can ignore
}

pub async fn ollama_chat_handler(
    State(_state): State<AppState>, // Assuming you have AppState
    Json(payload): Json<ChatApiRequest>,
) -> Result<impl IntoResponse, AppError> {

    // Construct a detailed prompt for the LLM
    let prompt = format!(
        "You are a helpful writing assistant. The user is currently editing a document with the following content:\n\n---\n{}\n---\n\nNow, please answer the user's question: {}",
        payload.context,
        payload.message
    );

    let ollama_request = OllamaRequest {
        model: OLLAMA_MODEL.to_string(),
        prompt,
        stream: false,
    };
    println!("Ollama Request Prompt: {}", ollama_request.prompt);

    let client = reqwest::Client::new();
    let res = client.post(format!("{}/api/generate", OLLAMA_ADDR))
        .header("Content-Type", "application/json")
        .json(&ollama_request)
        .send()
        .await
        .map_err(|_e| AppError::Llm("Failed to send request to LLM".into()))?;

    if res.status().is_success() {
        let ollama_response = res.json::<OllamaResponse>().await.map_err(|_| AppError::Llm("Failed to parse LLM response".into()))?;
        Ok(Json(json!({ "reply": ollama_response.response })))
    } else {
        Err(AppError::Llm(format!("LLM request failed: {}", res.status())))
    }
}
