use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::info;

use crate::types::{AgentEvent, BotConfig};

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

        let env: Vec<(&str, &str)> = vec![];
        let work_dir = config.working_dir.as_deref();

        info!("Spawning Codex: {} {} (cwd: {:?})", self.codex_path, args.join(" "), work_dir);
        let handle = ProcessHandle::spawn(&self.codex_path, &args, &env, work_dir).await?;
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
}
