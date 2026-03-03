use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::info;

use crate::types::{AgentEvent, BotConfig, EngineCapabilities};

use super::{AgentEngine, ProcessHandle};

pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        Self
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

        // Add system prompt if provided
        let system_prompt_owned;
        if let Some(ref prompt) = config.system_prompt {
            system_prompt_owned = prompt.clone();
            args.push("--system-prompt");
            args.push(&system_prompt_owned);
        }

        // Add working directory
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

        info!("Spawning Claude Code: claude {}", args.join(" "));
        let handle = ProcessHandle::spawn("claude", &args, &env).await?;
        info!("Claude Code process started, pid={}", handle.pid);

        Ok(handle)
    }

    async fn send(&self, handle: &ProcessHandle, message: &str) -> Result<()> {
        // stream-json input format expects JSON messages
        let msg = serde_json::json!({
            "type": "user_message",
            "content": message,
        });
        handle.send_line(&serde_json::to_string(&msg)?).await
    }

    fn subscribe(&self, handle: &ProcessHandle) -> Result<mpsc::Receiver<AgentEvent>> {
        handle
            .take_event_rx()
            .ok_or_else(|| anyhow::anyhow!("Event receiver already taken"))
    }

    async fn stop(&self, handle: &ProcessHandle) -> Result<()> {
        handle.stop().await
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            name: "Claude Code".to_string(),
            supports_streaming: true,
            supports_tools: true,
            supports_thinking: true,
            supports_images: true,
        }
    }
}
