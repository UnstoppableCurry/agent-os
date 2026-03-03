use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::info;

use crate::types::{AgentEvent, BotConfig};

use super::{AgentEngine, ProcessHandle};

pub struct ClaudeCodeAdapter {
    claude_path: String,
}

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        let claude_path = std::env::var("CLAUDE_PATH").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            let candidates = [
                format!("{}/npm-global/bin/claude", home),
                format!("{}/.npm-global/bin/claude", home),
                "/usr/local/bin/claude".to_string(),
                "claude".to_string(),
            ];
            for c in &candidates {
                if std::path::Path::new(c).exists() {
                    return c.clone();
                }
            }
            "claude".to_string()
        });
        info!("Claude binary path: {}", claude_path);
        Self { claude_path }
    }
}

#[async_trait]
impl AgentEngine for ClaudeCodeAdapter {
    async fn spawn(&self, config: &BotConfig) -> Result<ProcessHandle> {
        let mut args = vec![
            "--output-format", "stream-json",
            "--input-format", "stream-json",
            "--verbose",
            "--dangerously-skip-permissions",
        ];

        let system_prompt_owned;
        if let Some(ref prompt) = config.system_prompt {
            system_prompt_owned = prompt.clone();
            args.push("--system-prompt");
            args.push(&system_prompt_owned);
        }

        let env = vec![
            ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
            ("CLAUDE_CODE_DISABLE_BACKGROUND_TASKS", "1"),
        ];

        let work_dir = config.working_dir.as_deref();
        info!("Spawning Claude Code: {} {} (cwd: {:?})", self.claude_path, args.join(" "), work_dir);
        let handle = ProcessHandle::spawn(&self.claude_path, &args, &env, work_dir).await?;
        info!("Claude Code process started, pid={}", handle.pid);

        // Send the SDK initialization control request
        // Claude Code's stream-json input requires this before accepting user messages
        let init_msg = serde_json::json!({
            "type": "control_request",
            "request": {
                "subtype": "initialize"
            },
            "request_id": "init-1"
        });
        info!("Sending SDK initialize request to pid={}", handle.pid);
        handle.send_line(&serde_json::to_string(&init_msg)?).await?;

        Ok(handle)
    }

    async fn send(&self, handle: &ProcessHandle, message: &str) -> Result<()> {
        // stream-json user message format (verified from Claude Agent SDK)
        let msg = serde_json::json!({
            "type": "user",
            "session_id": "",
            "message": {
                "role": "user",
                "content": message,
            },
            "parent_tool_use_id": null,
        });
        handle.send_line(&serde_json::to_string(&msg)?).await
    }

    fn subscribe(&self, handle: &ProcessHandle) -> broadcast::Receiver<AgentEvent> {
        handle.subscribe()
    }

    async fn stop(&self, handle: &ProcessHandle) -> Result<()> {
        handle.stop().await
    }
}
