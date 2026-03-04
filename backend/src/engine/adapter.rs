use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::types::{AgentEvent, BotConfig, PermissionMode};

use super::ProcessHandle;

#[async_trait]
pub trait AgentEngine: Send + Sync {
    /// Spawn a CLI process for this engine
    async fn spawn(&self, config: &BotConfig) -> Result<ProcessHandle>;

    /// Spawn with session resumption and permission mode
    async fn spawn_with_options(
        &self,
        config: &BotConfig,
        session_id: Option<&str>,
        permission_mode: Option<&PermissionMode>,
    ) -> Result<ProcessHandle> {
        // Default: just call spawn (adapters override for specific behavior)
        let _ = session_id;
        let _ = permission_mode;
        self.spawn(config).await
    }

    /// Send a message to the process stdin
    async fn send(&self, handle: &ProcessHandle, message: &str) -> Result<()>;

    /// Subscribe to events from the process stdout
    fn subscribe(&self, handle: &ProcessHandle) -> broadcast::Receiver<AgentEvent>;

    /// Stop the process
    async fn stop(&self, handle: &ProcessHandle) -> Result<()>;
}
