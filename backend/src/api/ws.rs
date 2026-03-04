use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::bot::BotManager;
use crate::types::AgentEvent;

pub async fn handle_socket_inner(mut socket: WebSocket, mgr: Arc<BotManager>, bot_id: Uuid) {
    info!("WebSocket connected for bot {}", bot_id);

    // Replay buffered events for history
    let history = mgr.get_buffered_events(bot_id).await;
    if !history.is_empty() {
        info!("Replaying {} buffered events for bot {}", history.len(), bot_id);
        for event in &history {
            if let Some(text) = event_to_text(event) {
                if socket.send(Message::Text(text.into())).await.is_err() {
                    return;
                }
            }
        }
    }

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
                        // Extract session_id from init events
                        if let Some(sid) = BotManager::extract_session_id(&event) {
                            mgr.set_session_id(bot_id, sid).await;
                        }

                        // Log event to session store
                        mgr.log_event(bot_id, &event).await;

                        let text = event_to_text(&event);
                        debug!("WS event: {:?} -> {:?}", event, text);
                        if let Some(text) = text {
                            if socket.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
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
                        debug!("WS received from client: {}", text);
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

/// Convert Claude Code stream-json events to display text
fn event_to_text(event: &AgentEvent) -> Option<String> {
    match event {
        AgentEvent::Raw { text } => Some(text.clone()),

        AgentEvent::System { subtype, model, cwd, .. } => {
            match subtype.as_deref() {
                Some("init") => {
                    let model_str = model.as_deref().unwrap_or("unknown");
                    let cwd_str = cwd.as_deref().unwrap_or(".");
                    Some(format!("🟢 已就绪 | 模型: {} | 目录: {}", model_str, cwd_str))
                }
                // Skip hook events (noise)
                Some("hook_started") | Some("hook_response") => None,
                _ => None,
            }
        }

        AgentEvent::Assistant { message, .. } => {
            // Extract text from message.content[]
            let content = message.get("content")?.as_array()?;
            let mut parts = Vec::new();

            for block in content {
                let block_type = block.get("type")?.as_str()?;
                match block_type {
                    "text" => {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            if !text.is_empty() {
                                parts.push(text.to_string());
                            }
                        }
                    }
                    "tool_use" => {
                        let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                        let input = block.get("input").cloned().unwrap_or(serde_json::Value::Null);
                        let input_str = serde_json::to_string(&input).unwrap_or_default();
                        let truncated = if input_str.len() > 200 {
                            format!("{}...", &input_str[..200])
                        } else {
                            input_str
                        };
                        parts.push(format!("🔧 {} | {}", name, truncated));
                    }
                    "tool_result" => {
                        if let Some(content_val) = block.get("content") {
                            let result_text = if let Some(s) = content_val.as_str() {
                                s.to_string()
                            } else {
                                serde_json::to_string(content_val).unwrap_or_default()
                            };
                            let truncated = if result_text.len() > 500 {
                                format!("{}...", &result_text[..500])
                            } else {
                                result_text
                            };
                            parts.push(format!("📋 {}", truncated));
                        }
                    }
                    "thinking" => {
                        if let Some(thinking) = block.get("thinking").and_then(|t| t.as_str()) {
                            if !thinking.is_empty() {
                                let truncated = if thinking.len() > 300 {
                                    format!("{}...", &thinking[..300])
                                } else {
                                    thinking.to_string()
                                };
                                parts.push(format!("💭 {}", truncated));
                            }
                        }
                    }
                    _ => {}
                }
            }

            if parts.is_empty() {
                None
            } else {
                Some(parts.join("\n"))
            }
        }

        AgentEvent::Result { result, subtype, duration_ms, is_error, .. } => {
            if is_error == &Some(true) {
                let err_text = result.as_deref().unwrap_or("未知错误");
                return Some(format!("❌ 错误: {}", err_text));
            }
            let duration = duration_ms.map(|ms| format!(" ({}ms)", ms)).unwrap_or_default();
            match subtype.as_deref() {
                Some("success") => {
                    Some(format!("\n───{}\n", duration))
                }
                _ => None,
            }
        }

        AgentEvent::User { .. } => None,

        AgentEvent::ControlResponse { .. } => None,

        AgentEvent::Unknown => None,
    }
}

async fn recv_msg(socket: &mut WebSocket) -> Option<Message> {
    match socket.recv().await {
        Some(Ok(msg)) => Some(msg),
        _ => None,
    }
}
