use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::info;

use crate::types::{AgentEvent, BotConfig, PermissionMode};

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

    fn build_args(
        config: &BotConfig,
        session_id: Option<&str>,
        permission_mode: Option<&PermissionMode>,
    ) -> Vec<String> {
        let mut args = vec![
            "--output-format".to_string(), "stream-json".to_string(),
            "--input-format".to_string(), "stream-json".to_string(),
            "--verbose".to_string(),
        ];

        // Permission mode
        match permission_mode {
            Some(PermissionMode::Default) => {
                // No permission flags
            }
            Some(PermissionMode::AcceptEdits) => {
                args.push("--permission-mode".to_string());
                args.push("acceptEdits".to_string());
            }
            Some(PermissionMode::Plan) => {
                args.push("--permission-mode".to_string());
                args.push("plan".to_string());
            }
            Some(PermissionMode::BypassPermissions) | None => {
                args.push("--dangerously-skip-permissions".to_string());
            }
        }

        // Session resumption
        if let Some(sid) = session_id {
            args.push("--session-id".to_string());
            args.push(sid.to_string());
            args.push("--resume".to_string());
        }

        // System prompt
        if let Some(ref prompt) = config.system_prompt {
            args.push("--system-prompt".to_string());
            args.push(prompt.clone());
        }

        args
    }
}

#[async_trait]
impl AgentEngine for ClaudeCodeAdapter {
    async fn spawn(&self, config: &BotConfig) -> Result<ProcessHandle> {
        self.spawn_with_options(config, None, None).await
    }

    async fn spawn_with_options(
        &self,
        config: &BotConfig,
        session_id: Option<&str>,
        permission_mode: Option<&PermissionMode>,
    ) -> Result<ProcessHandle> {
        let args_owned = Self::build_args(config, session_id, permission_mode);
        let args: Vec<&str> = args_owned.iter().map(|s| s.as_str()).collect();

        let env = vec![
            ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
            ("CLAUDE_CODE_DISABLE_BACKGROUND_TASKS", "1"),
        ];

        let work_dir = config.working_dir.as_deref();
        info!("Spawning Claude Code: {} {} (cwd: {:?})", self.claude_path, args.join(" "), work_dir);
        let handle = ProcessHandle::spawn(&self.claude_path, &args, &env, work_dir).await?;
        info!("Claude Code process started, pid={}", handle.pid);

        // Send the SDK initialization control request (only for new sessions)
        if session_id.is_none() {
            let init_msg = serde_json::json!({
                "type": "control_request",
                "request": {
                    "subtype": "initialize"
                },
                "request_id": "init-1"
            });
            info!("Sending SDK initialize request to pid={}", handle.pid);
            handle.send_line(&serde_json::to_string(&init_msg)?).await?;
        }

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
