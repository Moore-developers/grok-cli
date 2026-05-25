use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use ax_extract::State;
use axum::extract as ax_extract;
use axum::http::StatusCode;
use axum::response::{sse, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Router, Json};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::app::AppContext;
use crate::upstream;

// ─── OpenAI-compatible request/response structures ─────────────────────────

#[derive(Debug, serde::Deserialize, Clone)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub stop: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionResponseChoice>,
    pub usage: CompletionUsage,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct ChatCompletionResponseChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct CompletionUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChunkChoice>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct ChatCompletionChunkChoice {
    pub index: u32,
    pub delta: MessageDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct MessageDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenAIError {
    pub error: OpenAIErrorDetail,
}

#[derive(Debug, serde::Serialize)]
pub struct OpenAIErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
}

// ─── Application State ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub ctx: AppContext,
    pub default_model: String,
}

// ─── Middleware: catch panics and convert to 500 ───────────────────────────

fn map_app_error_to_status(err: &crate::error::AppError) -> StatusCode {
    use crate::error::ErrorCode;
    match err.code {
        ErrorCode::AuthMissing | ErrorCode::AuthExpired | ErrorCode::AuthReloginRequired | ErrorCode::AuthRefreshFailed => StatusCode::UNAUTHORIZED,
        ErrorCode::InvalidArgs => StatusCode::BAD_REQUEST,
        ErrorCode::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        ErrorCode::ModelCapabilityMismatch => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// ─── Handler: POST /v1/chat/completions ────────────────────────────────────

async fn chat_completions_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatCompletionRequest>,
) -> Response {
    let model = req.model.clone().unwrap_or_else(|| state.default_model.clone());
    let stream = req.stream.unwrap_or(false);

    let grok_body = build_grok_request(&req, &model);

    if stream {
        let ctx = state.ctx.clone();
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                upstream::post_responses_stream_api(&ctx, None, &grok_body, Some(3600))
            }).await;

            match result {
                Ok(Ok(stream_env)) => {
                    let text = stream_env.response.text().unwrap_or_default();
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                let _ = tx.send(Ok::<_, std::convert::Infallible>(
                                    sse::Event::default().data("data: [DONE]\n\n")
                                )).await;
                                break;
                            }
                            if let Some(chunk) = convert_grok_sse_to_openai_chunk(data, &model) {
                                let _ = tx.send(Ok(sse::Event::default()
                                    .data(serde_json::to_string(&chunk).unwrap() + "\n\n")
                                )).await;
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    let msg = format!("upstream error: {}", e.message);
                    let _ = tx.send(Ok(sse::Event::default()
                        .data(format!("data: {{\"error\": {{\"message\": \"{}\", \"type\": \"upstream_error\"}}}}\n\n", msg))
                    )).await;
                }
                Err(e) => {
                    let _ = tx.send(Ok(sse::Event::default()
                        .data(format!("data: {{\"error\": {{\"message\": \"runtime panic: {}\", \"type\": \"internal_error\"}}}}\n\n", e))
                    )).await;
                }
            }
        });

        let stream = ReceiverStream::new(rx);
        sse::Sse::new(stream).into_response()
    } else {
        let ctx = state.ctx.clone();

        let result = tokio::task::spawn_blocking(move || {
            upstream::post_responses_api(&ctx, None, &grok_body, Some(3600))
        }).await;

        match result {
            Ok(Ok(env)) => {
                let response = convert_grok_response_to_openai(&env.response, &model);
                Json(response).into_response()
            }
            Ok(Err(e)) => {
                let status = map_app_error_to_status(&e);
                let body = serde_json::to_string(&OpenAIError {
                    error: OpenAIErrorDetail {
                        message: e.message.clone(),
                        error_type: "upstream_error".to_string(),
                        code: Some(format!("{:?}", e.code)),
                    }
                }).unwrap();
                (status, body).into_response()
            }
            Err(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("panic: {}", e)).into_response()
            }
        }
    }
}

// ─── Handler: GET /v1/models ───────────────────────────────────────────────

async fn models_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "object": "list",
        "data": [
            {
                "id": "grok-1",
                "object": "model",
                "created": 1677610612,
                "owned_by": "xai",
                "permission": []
            },
            {
                "id": "grok-2",
                "object": "model",
                "created": 1677610612,
                "owned_by": "xai",
                "permission": []
            },
            {
                "id": "grok-4.3",
                "object": "model",
                "created": 1715729000,
                "owned_by": "xai",
                "permission": []
            }
        ]
    }))
}

// ─── Handler: GET /health ─────────────────────────────────────────────────

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "grok-cli-serve"}))
}

// ─── Request/response conversion ───────────────────────────────────────────

fn build_grok_request(req: &ChatCompletionRequest, model: &str) -> serde_json::Value {
    use serde_json::json;

    let mut messages: Vec<serde_json::Value> = Vec::new();
    for msg in &req.messages {
        messages.push(json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    let mut request = json!({
        "model": model,
        "messages": messages,
        "tools": [
            {
                "type": "web_search",
                "web_search": {}
            }
        ]
    });

    if let Some(temperature) = req.temperature {
        request["temperature"] = serde_json::json!(temperature);
    }
    if let Some(max_tokens) = req.max_tokens {
        request["max_tokens"] = serde_json::json!(max_tokens);
    }
    if let Some(top_p) = req.top_p {
        request["top_p"] = serde_json::json!(top_p);
    }
    if let Some(stop) = &req.stop {
        request["stop"] = stop.clone();
    }

    request
}

fn convert_grok_response_to_openai(grok_resp: &serde_json::Value, model: &str) -> ChatCompletionResponse {
    let output = grok_resp.get("output").and_then(|o| o.as_array())
        .and_then(|arr| arr.iter().find(|item| item.get("type").and_then(|t| t.as_str()) == Some("message")));

    let content_text = output
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.iter().find(|item| item.get("type").and_then(|t| t.as_str()) == Some("output_text")))
        .and_then(|t| t.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let usage = grok_resp.get("usage")
        .map(|u| CompletionUsage {
            prompt_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            completion_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        })
        .unwrap_or(CompletionUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });

    ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid_simple()),
        object: "chat.completion".to_string(),
        created: unix_timestamp(),
        model: model.to_string(),
        choices: vec![ChatCompletionResponseChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: content_text,
            },
            finish_reason: "stop".to_string(),
        }],
        usage,
    }
}

fn convert_grok_sse_to_openai_chunk(grok_data: &str, model: &str) -> Option<ChatCompletionChunk> {
    let ok_value: serde_json::Value = serde_json::from_str(grok_data).ok()?;

    let text = ok_value
        .get("output")?.as_array()?
        .iter()
        .find_map(|item| {
            if item.get("type").and_then(|t| t.as_str()) != Some("content_block") {
                return None;
            }
            item.get("content")?.as_array()?
                .iter()
                .find_map(|c| {
                    if c.get("type").and_then(|t| t.as_str()) != Some("output_text") {
                        return None;
                    }
                    c.get("text")?.as_str().map(String::from)
                })
        })?;

    let done = ok_value.get("done").and_then(|d| d.as_bool()).unwrap_or(false);

    Some(ChatCompletionChunk {
        id: format!("chatcmpl-{}", uuid_simple()),
        object: "chat.completion.chunk".to_string(),
        created: unix_timestamp(),
        model: model.to_string(),
        choices: vec![ChatCompletionChunkChoice {
            index: 0,
            delta: MessageDelta {
                role: Some("assistant".to_string()),
                content: Some(text),
            },
            finish_reason: if done { Some("stop".to_string()) } else { None },
        }],
    })
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{:x}{:x}", now.as_secs(), now.subsec_nanos())
}

// ─── Main server bootstrap ──────────────────────────────────────────────────

pub async fn run(port: u16, default_model: String, ctx: AppContext) -> anyhow::Result<()> {
    let app_state = Arc::new(AppState {
        ctx,
        default_model,
    });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/v1/models", get(models_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 grok-cli serve listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await.context("server error")
}