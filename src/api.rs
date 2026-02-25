use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::trace::TraceLayer;

use crate::{config::AppConfig, AuriaAgent, models::ChatCompletionRequest};

#[derive(Clone)]
struct ApiState {
    agent: AuriaAgent,
}

pub async fn serve(cfg: AppConfig, agent: AuriaAgent) -> anyhow::Result<()> {
    let state = ApiState { agent };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/chat/completions", post(chat_completions))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn chat_completions(
    State(st): State<ApiState>,
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    match st.agent.chat_completions(req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => {
            let body = serde_json::json!({ "error": { "message": e.to_string(), "type": "auria_error" }});
            (StatusCode::BAD_REQUEST, Json(body)).into_response()
        }
    }
}
