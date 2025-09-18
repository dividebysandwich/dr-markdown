
use serde::{Deserialize, Serialize};
use axum::{
    extract::State,
    response::IntoResponse,
    Json, body::Body
};

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
        stream: true,
    };
    println!("Ollama Request Prompt: {}", ollama_request.prompt);

    let client = reqwest::Client::new();
    let res = client.post(format!("{}/api/generate", OLLAMA_ADDR))
        .header("Content-Type", "application/json")
        .json(&ollama_request)
        .send()
        .await
        .map_err(|_e| AppError::Llm("Failed to send request to LLM".into()))?;


    // Get the response body as a stream of bytes
    let stream = res.bytes_stream();

    // Create a streaming body for the Axum response
    let body = Body::from_stream(stream);

    // Set the content type for a streaming response
    let headers = [
        (axum::http::header::CONTENT_TYPE, "text/event-stream"),
    ];

    Ok((headers, body))

}
