mod api;
mod bot;
mod engine;
mod memory;
mod sensor;
mod skill;
mod types;

use std::sync::Arc;

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, routing::{get, post}};
use tower_http::cors::CorsLayer;
use tracing::info;

use bot::BotManager;
use memory::{CrystalStore, StreamStore};

/// Shared application state
#[derive(Clone)]
struct AppState {
    stream_store: Arc<StreamStore>,
    crystal_store: Arc<CrystalStore>,
    bot_manager: Arc<BotManager>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agent_os=info,tower_http=info".parse().unwrap()),
        )
        .init();

    let memory_dir = std::env::var("AGENT_OS_MEMORY_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/projects/agent-os/memory", home)
    });

    info!("Memory directory: {}", memory_dir);

    let state = AppState {
        stream_store: Arc::new(StreamStore::new(format!("{}/stream", memory_dir))),
        crystal_store: Arc::new(CrystalStore::new(format!("{}/crystal", memory_dir))),
        bot_manager: Arc::new(BotManager::new()),
    };

    let app = Router::new()
        .route("/health", get(api::health::health))
        .route("/v1/bots", post(bots_create).get(bots_list))
        .route("/v1/bots/{id}", get(bots_get).delete(bots_stop))
        .route("/v1/bots/{id}/messages", post(bots_send))
        .route("/v1/bots/{id}/ws", get(ws_upgrade))
        .route("/v1/events", post(events_ingest))
        .route("/v1/memory/crystals", get(crystals_list))
        .route("/v1/memory/crystals/{name}", get(crystals_get))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("AgentOS backend starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ─── Bot Handlers ───

async fn bots_create(
    State(s): State<AppState>,
    Json(req): Json<types::CreateBotRequest>,
) -> Response {
    match s.bot_manager.create(req).await {
        Ok(status) => (StatusCode::CREATED, Json(types::ApiResponse::ok(status))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(types::ApiResponse::<()>::err(e.to_string()))).into_response(),
    }
}

async fn bots_list(State(s): State<AppState>) -> Json<types::ApiResponse<Vec<types::BotStatus>>> {
    Json(types::ApiResponse::ok(s.bot_manager.list().await))
}

async fn bots_get(State(s): State<AppState>, Path(id): Path<uuid::Uuid>) -> Response {
    match s.bot_manager.get(id).await {
        Some(st) => (StatusCode::OK, Json(types::ApiResponse::ok(st))).into_response(),
        None => (StatusCode::NOT_FOUND, Json(types::ApiResponse::<()>::err("Bot not found"))).into_response(),
    }
}

async fn bots_stop(State(s): State<AppState>, Path(id): Path<uuid::Uuid>) -> Response {
    match s.bot_manager.stop(id).await {
        Ok(_) => Json(types::ApiResponse::ok("stopped")).into_response(),
        Err(e) => Json(types::ApiResponse::<()>::err(e.to_string())).into_response(),
    }
}

async fn bots_send(
    State(s): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<types::SendMessageRequest>,
) -> Response {
    match s.bot_manager.send_message(id, &req.content).await {
        Ok(_) => Json(types::ApiResponse::ok("sent")).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(types::ApiResponse::<()>::err(e.to_string()))).into_response(),
    }
}

// ─── WebSocket ───

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(s): State<AppState>,
    Path(bot_id): Path<uuid::Uuid>,
) -> Response {
    ws.on_upgrade(move |socket| api::ws::handle_socket_inner(socket, s.bot_manager, bot_id))
}

// ─── Events ───

async fn events_ingest(
    State(s): State<AppState>,
    Json(req): Json<types::IngestEventsRequest>,
) -> Response {
    let source = format!("app:{}", req.app_id);
    let mut count = 0u32;
    for raw in req.events {
        let event = types::StreamEvent {
            ts: raw.ts.unwrap_or_else(chrono::Utc::now),
            source: source.clone(),
            event_type: raw.event_type,
            data: raw.data,
            meta: raw.meta,
        };
        if let Err(e) = s.stream_store.append(&event).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(types::ApiResponse::<()>::err(e.to_string()))).into_response();
        }
        count += 1;
    }
    Json(types::ApiResponse::ok(serde_json::json!({"ingested": count}))).into_response()
}

// ─── Memory ───

async fn crystals_list(State(s): State<AppState>) -> Response {
    match s.crystal_store.list().await {
        Ok(names) => Json(types::ApiResponse::ok(names)).into_response(),
        Err(e) => Json(types::ApiResponse::<()>::err(e.to_string())).into_response(),
    }
}

async fn crystals_get(State(s): State<AppState>, Path(name): Path<String>) -> Response {
    match s.crystal_store.read(&name).await {
        Ok(Some(content)) => Json(types::ApiResponse::ok(serde_json::json!({"name": name, "content": content}))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(types::ApiResponse::<()>::err("Crystal not found"))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(types::ApiResponse::<()>::err(e.to_string()))).into_response(),
    }
}
