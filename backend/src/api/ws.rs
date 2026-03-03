use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use tracing::{error, info};
use uuid::Uuid;

use crate::bot::BotManager;
use crate::types::AgentEvent;

pub async fn handle_socket_inner(mut socket: WebSocket, mgr: Arc<BotManager>, bot_id: Uuid) {
    info!("WebSocket connected for bot {}", bot_id);

    let mut event_rx = match mgr.subscribe(bot_id).await {
        Ok(rx) => rx,
        Err(e) => {
            error!("Failed to subscribe to bot {}: {}", bot_id, e);
            let _ = socket
                .send(Message::Text(
                    format!("[错误] {}", e).into(),
                ))
                .await;
            return;
        }
    };

    loop {
        tokio::select! {
            result = event_rx.recv() => {
                match result {
                    Ok(event) => {
                        // For raw events, send plain text; for others, serialize
                        let text = match &event {
                            AgentEvent::Raw { text } => text.clone(),
                            other => serde_json::to_string(other).unwrap_or_default(),
                        };
                        if socket.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        info!("WebSocket subscriber lagged {} messages", n);
                        continue;
                    }
                    Err(_) => break,
                }
            }
            Some(msg) = recv_msg(&mut socket) => {
                match msg {
                    Message::Text(text) => {
                        // Send raw text to bot's stdin
                        if let Err(e) = mgr.send_stdin(bot_id, &text).await {
                            error!("Failed to send to bot stdin: {}", e);
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            else => break,
        }
    }

    info!("WebSocket disconnected for bot {}", bot_id);
}

async fn recv_msg(socket: &mut WebSocket) -> Option<Message> {
    match socket.recv().await {
        Some(Ok(msg)) => Some(msg),
        _ => None,
    }
}
