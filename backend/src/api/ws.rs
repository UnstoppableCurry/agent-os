use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use tracing::{error, info};
use uuid::Uuid;

use crate::bot::BotManager;

pub async fn handle_socket_inner(mut socket: WebSocket, mgr: Arc<BotManager>, bot_id: Uuid) {
    info!("WebSocket connected for bot {}", bot_id);

    let mut event_rx = match mgr.subscribe(bot_id).await {
        Ok(rx) => rx,
        Err(e) => {
            error!("Failed to subscribe to bot {}: {}", bot_id, e);
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({"error": e.to_string()}).to_string().into(),
                ))
                .await;
            return;
        }
    };

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                let json = match serde_json::to_string(&event) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("Failed to serialize event: {}", e);
                        continue;
                    }
                };
                if socket.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
            Some(msg) = recv_msg(&mut socket) => {
                match msg {
                    Message::Text(text) => {
                        if let Err(e) = mgr.send_message(bot_id, &text).await {
                            error!("Failed to send message to bot: {}", e);
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
