use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::info;

use crate::types::{AgentEvent, BotConfig, EngineCapabilities};

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

        let work_dir;
        if let Some(ref dir) = config.working_dir {
            work_dir = dir.clone();
            args.push("--cwd");
            args.push(&work_dir);
        }

        let env = vec![
            ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
            ("CLAUDE_CODE_DISABLE_BACKGROUND_TASKS", "1"),
        ];

        info!("Spawning Claude Code: {} {}", self.claude_path, args.join(" "));
        let handle = ProcessHandle::spawn(&self.claude_path, &args, &env).await?;
        info!("Claude Code process started, pid={}", handle.pid);

        Ok(handle)
    }

    async fn send(&self, handle: &ProcessHandle, message: &str) -> Result<()> {
        // stream-json input format
        let msg = serde_json::json!({
            "type": "user_message",
            "content": message,
        });
        handle.send_line(&serde_json::to_string(&msg)?).await
    }

    fn subscribe(&self, handle: &ProcessHandle) -> broadcast::Receiver<AgentEvent> {
        handle.subscribe()
    }

    async fn stop(&self, handle: &ProcessHandle) -> Result<()> {
        handle.stop().await
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            name: "Claude Code".to_string(),
            supports_streaming: true,
            supports_tools: true,
        }
    }
}
