use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::types::{AgentEvent, BotConfig, EngineCapabilities};

use super::ProcessHandle;

#[async_trait]
pub trait AgentEngine: Send + Sync {
    /// Spawn a CLI process for this engine
    async fn spawn(&self, config: &BotConfig) -> Result<ProcessHandle>;

    /// Send a message to the process stdin
    async fn send(&self, handle: &ProcessHandle, message: &str) -> Result<()>;

    /// Subscribe to events from the process stdout
    fn subscribe(&self, handle: &ProcessHandle) -> broadcast::Receiver<AgentEvent>;

    /// Stop the process
    async fn stop(&self, handle: &ProcessHandle) -> Result<()>;

    /// Engine capabilities
    fn capabilities(&self) -> EngineCapabilities;
}
