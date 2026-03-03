use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::info;

use crate::types::{AgentEvent, BotConfig, EngineCapabilities};

use super::{AgentEngine, ProcessHandle};

pub struct KimiAdapter {
    kimi_path: String,
}

impl KimiAdapter {
    pub fn new() -> Self {
        let kimi_path = std::env::var("KIMI_PATH").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            let candidates = [
                format!("{}/npm-global/bin/kimi", home),
                format!("{}/.npm-global/bin/kimi", home),
                "/usr/local/bin/kimi".to_string(),
                "kimi".to_string(),
            ];
            for c in &candidates {
                if std::path::Path::new(c).exists() {
                    return c.clone();
                }
            }
            "kimi".to_string()
        });
        info!("Kimi binary path: {}", kimi_path);
        Self { kimi_path }
    }
}

#[async_trait]
impl AgentEngine for KimiAdapter {
    async fn spawn(&self, config: &BotConfig) -> Result<ProcessHandle> {
        let mut args = vec![
            "--output-format", "stream-json",
            "--input-format", "stream-json",
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

        let env: Vec<(&str, &str)> = vec![];

        info!("Spawning Kimi: {} {}", self.kimi_path, args.join(" "));
        let handle = ProcessHandle::spawn(&self.kimi_path, &args, &env).await?;
        info!("Kimi process started, pid={}", handle.pid);

        Ok(handle)
    }

    async fn send(&self, handle: &ProcessHandle, message: &str) -> Result<()> {
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
            name: "Kimi".to_string(),
            supports_streaming: true,
            supports_tools: true,
        }
    }
}
