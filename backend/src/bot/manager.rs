use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use tokio::sync::{Mutex, RwLock, broadcast};
use tracing::{info, error};
use uuid::Uuid;

use crate::engine::{AgentEngine, ClaudeCodeAdapter, CodexAdapter, KimiAdapter, ProcessHandle};
use crate::types::*;

struct BotInstance {
    config: BotConfig,
    status: BotStatus,
    handle: Option<ProcessHandle>,
}

pub struct BotManager {
    bots: RwLock<HashMap<Uuid, Arc<Mutex<BotInstance>>>>,
    engines: HashMap<String, Box<dyn AgentEngine>>,
}

impl BotManager {
    pub fn new() -> Self {
        let mut engines: HashMap<String, Box<dyn AgentEngine>> = HashMap::new();
        engines.insert("claude".to_string(), Box::new(ClaudeCodeAdapter::new()));
        engines.insert("kimi".to_string(), Box::new(KimiAdapter::new()));
        engines.insert("codex".to_string(), Box::new(CodexAdapter::new()));

        Self {
            bots: RwLock::new(HashMap::new()),
            engines,
        }
    }

    pub async fn create(&self, req: CreateBotRequest) -> Result<BotStatus> {
        let id = Uuid::new_v4();
        let config = BotConfig {
            id,
            name: req.name.clone(),
            engine: req.engine.clone(),
            role: req.role.clone(),
            system_prompt: req.system_prompt,
            skills: req.skills,
            working_dir: req.working_dir,
        };

        let status = BotStatus {
            id,
            name: req.name,
            engine: req.engine,
            role: req.role,
            state: BotState::Starting,
            created_at: Utc::now(),
            message_count: 0,
        };

        let engine_key = match &config.engine {
            EngineType::Claude => "claude",
            EngineType::Kimi => "kimi",
            EngineType::Codex => "codex",
        };

        let engine = self.engines.get(engine_key)
            .ok_or_else(|| anyhow::anyhow!("Engine '{}' not implemented", engine_key))?;

        let handle = match engine.spawn(&config).await {
            Ok(h) => h,
            Err(e) => {
                error!("Failed to spawn bot: {}", e);
                let mut failed_status = status.clone();
                failed_status.state = BotState::Error;
                return Ok(failed_status);
            }
        };

        let mut final_status = status;
        final_status.state = BotState::Running;

        let instance = BotInstance {
            config,
            status: final_status.clone(),
            handle: Some(handle),
        };

        self.bots.write().await.insert(id, Arc::new(Mutex::new(instance)));
        info!("Bot created: {} ({})", final_status.name, id);

        Ok(final_status)
    }

    pub async fn list(&self) -> Vec<BotStatus> {
        let bots = self.bots.read().await;
        let mut result = vec![];
        for bot in bots.values() {
            result.push(bot.lock().await.status.clone());
        }
        result
    }

    pub async fn get(&self, id: Uuid) -> Option<BotStatus> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)?;
        Some(bot.lock().await.status.clone())
    }

    /// Send a message and return a broadcast receiver for streaming events
    pub async fn send_message(&self, id: Uuid, content: &str) -> Result<broadcast::Receiver<AgentEvent>> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Bot not found"))?;

        let mut instance = bot.lock().await;

        let engine_key = match &instance.config.engine {
            EngineType::Claude => "claude",
            EngineType::Kimi => "kimi",
            EngineType::Codex => "codex",
        };

        let engine = self.engines.get(engine_key).unwrap();

        if let Some(ref handle) = instance.handle {
            // Subscribe BEFORE sending so we don't miss events
            let rx = engine.subscribe(handle);
            engine.send(handle, content).await?;
            instance.status.message_count += 1;
            Ok(rx)
        } else {
            Err(anyhow::anyhow!("Bot process not running"))
        }
    }

    /// Send raw text to bot's stdin (for WebSocket terminal)
    pub async fn send_stdin(&self, id: Uuid, text: &str) -> Result<()> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Bot not found"))?;

        let mut instance = bot.lock().await;

        if let Some(ref handle) = instance.handle {
            handle.send_line(text).await?;
            instance.status.message_count += 1;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Bot process not running"))
        }
    }

    /// Subscribe to bot events (for WebSocket)
    pub async fn subscribe(&self, id: Uuid) -> Result<broadcast::Receiver<AgentEvent>> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Bot not found"))?;

        let instance = bot.lock().await;

        if let Some(ref handle) = instance.handle {
            Ok(handle.subscribe())
        } else {
            Err(anyhow::anyhow!("Bot process not running"))
        }
    }

    pub async fn stop(&self, id: Uuid) -> Result<()> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Bot not found"))?;

        let mut instance = bot.lock().await;

        if let Some(ref handle) = instance.handle {
            handle.stop().await?;
        }

        instance.status.state = BotState::Stopped;
        instance.handle = None;
        info!("Bot stopped: {} ({})", instance.status.name, id);

        Ok(())
    }
}
