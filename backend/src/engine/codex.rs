use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::info;

use crate::types::{AgentEvent, BotConfig, EngineCapabilities};

use super::{AgentEngine, ProcessHandle};

pub struct CodexAdapter {
    codex_path: String,
}

impl CodexAdapter {
    pub fn new() -> Self {
        let codex_path = std::env::var("CODEX_PATH").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            let candidates = [
                format!("{}/npm-global/bin/codex", home),
                format!("{}/.npm-global/bin/codex", home),
                "/usr/local/bin/codex".to_string(),
                "codex".to_string(),
            ];
            for c in &candidates {
                if std::path::Path::new(c).exists() {
                    return c.clone();
                }
            }
            "codex".to_string()
        });
        info!("Codex binary path: {}", codex_path);
        Self { codex_path }
    }
}

#[async_trait]
impl AgentEngine for CodexAdapter {
    async fn spawn(&self, config: &BotConfig) -> Result<ProcessHandle> {
        let mut args = vec![
            "--full-auto",
        ];

        let work_dir;
        if let Some(ref dir) = config.working_dir {
            work_dir = dir.clone();
            args.push("--cwd");
            args.push(&work_dir);
        }

        let env: Vec<(&str, &str)> = vec![];

        info!("Spawning Codex: {} {}", self.codex_path, args.join(" "));
        let handle = ProcessHandle::spawn(&self.codex_path, &args, &env).await?;
        info!("Codex process started, pid={}", handle.pid);

        Ok(handle)
    }

    async fn send(&self, handle: &ProcessHandle, message: &str) -> Result<()> {
        // Codex uses plain text stdin
        handle.send_line(message).await
    }

    fn subscribe(&self, handle: &ProcessHandle) -> broadcast::Receiver<AgentEvent> {
        handle.subscribe()
    }

    async fn stop(&self, handle: &ProcessHandle) -> Result<()> {
        handle.stop().await
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            name: "Codex".to_string(),
            supports_streaming: true,
            supports_tools: true,
        }
    }
}
