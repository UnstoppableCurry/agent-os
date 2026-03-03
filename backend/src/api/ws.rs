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
                .send(Message::Text(format!("[错误] {}", e).into()))
                .await;
            return;
        }
    };

    loop {
        tokio::select! {
            result = event_rx.recv() => {
                match result {
                    Ok(event) => {
                        // Convert structured events to plain text for terminal display
                        let text = event_to_text(&event);
                        if text.is_empty() {
                            continue;
                        }
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
                        // Send user input to bot's stdin via engine adapter
                        if let Err(e) = mgr.send_message(bot_id, &text).await {
                            error!("Failed to send to bot: {}", e);
                            let _ = socket
                                .send(Message::Text(format!("[错误] {}", e).into()))
                                .await;
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

/// Convert an AgentEvent to plain text for terminal display
fn event_to_text(event: &AgentEvent) -> String {
    match event {
        AgentEvent::Raw { text } => text.clone(),

        AgentEvent::ContentBlockDelta { delta, .. } => {
            // text_delta → actual response text
            if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                return text.to_string();
            }
            // thinking_delta → thinking indicator
            if let Some(thinking) = delta.get("thinking").and_then(|t| t.as_str()) {
                return format!("💭 {}", thinking);
            }
            // input_json_delta → tool input being built
            if let Some(partial) = delta.get("partial_json").and_then(|t| t.as_str()) {
                return partial.to_string();
            }
            String::new()
        }

        AgentEvent::ContentBlockStart { content_block, .. } => {
            let block_type = content_block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match block_type {
                "tool_use" => {
                    let name = content_block.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                    format!("\n🔧 {}\n", name)
                }
                "thinking" => "\n💭 思考中...\n".to_string(),
                _ => String::new(),
            }
        }

        AgentEvent::Result { result, subtype } => {
            if subtype.as_deref() == Some("raw") {
                return result.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
            }
            // Tool results
            if let Some(content) = result.get("content").and_then(|c| c.as_str()) {
                let truncated = if content.len() > 500 {
                    format!("{}...(截断)", &content[..500])
                } else {
                    content.to_string()
                };
                return format!("📋 {}\n", truncated);
            }
            String::new()
        }

        AgentEvent::MessageStop { .. } => "\n───\n".to_string(),

        _ => String::new(),
    }
}

async fn recv_msg(socket: &mut WebSocket) -> Option<Message> {
    match socket.recv().await {
        Some(Ok(msg)) => Some(msg),
        _ => None,
    }
}
