use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio::task::JoinHandle;
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::bot::event_buffer::EventBuffer;
use crate::bot::session_store::SessionStore;
use crate::bot::supervisor::{ProcessSupervisor, SupervisorEvent};
use crate::engine::{AgentEngine, ClaudeCodeAdapter, CodexAdapter, KimiAdapter, ProcessHandle};
use crate::types::*;

struct BotInstance {
    config: BotConfig,
    status: BotStatus,
    handle: Arc<Mutex<Option<ProcessHandle>>>,
    supervisor_handle: Option<JoinHandle<()>>,
    /// Shared with ProcessSupervisor so restarts can use --resume
    session_id: Arc<RwLock<Option<String>>>,
    permission_mode: Option<PermissionMode>,
    idle_timeout_mins: Option<u32>,
    event_buffer: EventBuffer,
    last_active_at: chrono::DateTime<Utc>,
}

pub struct BotManager {
    bots: Arc<RwLock<HashMap<Uuid, Arc<Mutex<BotInstance>>>>>,
    engines: HashMap<String, Arc<dyn AgentEngine>>,
    session_store: Arc<SessionStore>,
}

impl BotManager {
    pub async fn new(data_dir: &str) -> Result<Self> {
        let mut engines: HashMap<String, Arc<dyn AgentEngine>> = HashMap::new();
        engines.insert("claude".to_string(), Arc::new(ClaudeCodeAdapter::new()));
        engines.insert("kimi".to_string(), Arc::new(KimiAdapter::new()));
        engines.insert("codex".to_string(), Arc::new(CodexAdapter::new()));

        let session_store = Arc::new(SessionStore::new(data_dir).await?);

        let manager = Self {
            bots: Arc::new(RwLock::new(HashMap::new())),
            engines,
            session_store,
        };

        // Restore bots from persisted state
        manager.restore_bots().await;

        Ok(manager)
    }

    /// Restore all bots that were Running or Suspended when the service last stopped
    async fn restore_bots(&self) {
        let records = match self.session_store.list_bots().await {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to load bot records: {}", e);
                return;
            }
        };

        for record in records {
            let id = record.config.id;
            let should_restore = matches!(
                record.status.state,
                BotState::Running | BotState::Suspended | BotState::Starting
            );

            if !should_restore {
                info!("Skipping restore for bot {} (state={:?})", id, record.status.state);
                continue;
            }

            info!("Restoring bot {} ({}) with session {:?}", record.config.name, id, record.session_id);

            let engine_key = Self::engine_key(&record.config.engine);
            let engine = match self.engines.get(engine_key) {
                Some(e) => e.clone(),
                None => {
                    error!("Engine '{}' not found for bot {}", engine_key, id);
                    continue;
                }
            };

            // Spawn with session resumption
            let handle = match engine.spawn_with_options(
                &record.config,
                record.session_id.as_deref(),
                record.permission_mode.as_ref(),
            ).await {
                Ok(h) => h,
                Err(e) => {
                    error!("Failed to restore bot {}: {}", id, e);
                    // Update persisted state to Error
                    let mut updated = record.clone();
                    updated.status.state = BotState::Error;
                    let _ = self.session_store.save_bot(&updated).await;
                    continue;
                }
            };

            let process_handle = Arc::new(Mutex::new(Some(handle)));

            // Shared session_id Arc (supervisor uses this when restarting)
            let session_id_arc: Arc<RwLock<Option<String>>> =
                Arc::new(RwLock::new(record.session_id.clone()));

            // Start supervisor
            let (sup_tx, sup_rx) = tokio::sync::mpsc::channel::<SupervisorEvent>(16);
            let supervisor_handle = ProcessSupervisor::start(
                id,
                record.config.clone(),
                process_handle.clone(),
                engine,
                sup_tx,
                session_id_arc.clone(),
                record.permission_mode.clone(),
            );

            let bots_ref = self.bots.clone();
            let store_ref = self.session_store.clone();
            tokio::spawn(async move {
                Self::handle_supervisor_events(id, sup_rx, bots_ref, store_ref).await;
            });

            let instance = BotInstance {
                config: record.config.clone(),
                status: BotStatus {
                    id,
                    name: record.config.name.clone(),
                    engine: record.config.engine.clone(),
                    role: record.config.role.clone(),
                    state: BotState::Running,
                    created_at: record.created_at,
                    message_count: record.message_count,
                },
                handle: process_handle,
                supervisor_handle: Some(supervisor_handle),
                session_id: session_id_arc,
                permission_mode: record.permission_mode,
                idle_timeout_mins: record.idle_timeout_mins,
                event_buffer: EventBuffer::new(500),
                last_active_at: Utc::now(),
            };

            self.bots.write().await.insert(id, Arc::new(Mutex::new(instance)));
            info!("Bot {} restored successfully", id);
        }
    }

    fn engine_key(engine: &EngineType) -> &'static str {
        match engine {
            EngineType::Claude => "claude",
            EngineType::Kimi => "kimi",
            EngineType::Codex => "codex",
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

        let engine_key = Self::engine_key(&config.engine);
        let engine = self.engines.get(engine_key)
            .ok_or_else(|| anyhow::anyhow!("Engine '{}' not implemented", engine_key))?
            .clone();

        let process_handle = match engine.spawn_with_options(
            &config,
            req.resume_session.as_deref(),
            req.permission_mode.as_ref(),
        ).await {
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

        let handle = Arc::new(Mutex::new(Some(process_handle)));

        // Shared session_id Arc (initially from resume_session, updated after init event)
        let session_id_arc: Arc<RwLock<Option<String>>> =
            Arc::new(RwLock::new(req.resume_session.clone()));

        // Start supervisor
        let (sup_tx, sup_rx) = tokio::sync::mpsc::channel::<SupervisorEvent>(16);
        let supervisor_handle = ProcessSupervisor::start(
            id,
            config.clone(),
            handle.clone(),
            engine,
            sup_tx,
            session_id_arc.clone(),
            req.permission_mode.clone(),
        );

        let bots_ref = self.bots.clone();
        let store_ref = self.session_store.clone();
        tokio::spawn(async move {
            Self::handle_supervisor_events(id, sup_rx, bots_ref, store_ref).await;
        });

        let instance = BotInstance {
            config: config.clone(),
            status: final_status.clone(),
            handle,
            supervisor_handle: Some(supervisor_handle),
            session_id: session_id_arc,
            permission_mode: req.permission_mode.clone(),
            idle_timeout_mins: req.idle_timeout_mins,
            event_buffer: EventBuffer::new(500),
            last_active_at: Utc::now(),
        };

        self.bots.write().await.insert(id, Arc::new(Mutex::new(instance)));
        info!("Bot created: {} ({})", final_status.name, id);

        // Persist to disk
        let record = BotRecord {
            config,
            status: final_status.clone(),
            session_id: None,
            permission_mode: req.permission_mode,
            idle_timeout_mins: req.idle_timeout_mins,
            created_at: final_status.created_at,
            last_active_at: Utc::now(),
            message_count: 0,
            restart_count: 0,
        };
        if let Err(e) = self.session_store.save_bot(&record).await {
            error!("Failed to persist bot record: {}", e);
        }

        Ok(final_status)
    }

    async fn handle_supervisor_events(
        bot_id: Uuid,
        mut rx: tokio::sync::mpsc::Receiver<SupervisorEvent>,
        bots: Arc<RwLock<HashMap<Uuid, Arc<Mutex<BotInstance>>>>>,
        store: Arc<SessionStore>,
    ) {
        while let Some(event) = rx.recv().await {
            match event {
                SupervisorEvent::Restarting { attempt, delay } => {
                    warn!("Bot {} restarting (attempt {}, delay {:?})", bot_id, attempt, delay);
                }
                SupervisorEvent::Restarted => {
                    info!("Bot {} process restarted successfully", bot_id);
                    // Update state back to Running
                    let bots_guard = bots.read().await;
                    if let Some(bot) = bots_guard.get(&bot_id) {
                        let mut instance = bot.lock().await;
                        instance.status.state = BotState::Running;
                    }
                }
                SupervisorEvent::RestartFailed(err) => {
                    error!("Bot {} restart failed: {}", bot_id, err);
                }
                SupervisorEvent::MaxRestartsExceeded => {
                    error!("Bot {} max restarts exceeded, marking as Error", bot_id);
                    let bots_guard = bots.read().await;
                    if let Some(bot) = bots_guard.get(&bot_id) {
                        let mut instance = bot.lock().await;
                        instance.status.state = BotState::Error;
                        // Update persisted state
                        let current_session_id = instance.session_id.read().await.clone();
                        let record = BotRecord {
                            config: instance.config.clone(),
                            status: instance.status.clone(),
                            session_id: current_session_id,
                            permission_mode: instance.permission_mode.clone(),
                            idle_timeout_mins: instance.idle_timeout_mins,
                            created_at: instance.status.created_at,
                            last_active_at: Utc::now(),
                            message_count: instance.status.message_count,
                            restart_count: 0,
                        };
                        let _ = store.save_bot(&record).await;
                    }
                }
            }
        }
    }

    /// Extract session_id from a System init event
    pub fn extract_session_id(event: &AgentEvent) -> Option<String> {
        if let AgentEvent::System { subtype, session_id, .. } = event {
            if subtype.as_deref() == Some("init") {
                return session_id.clone();
            }
        }
        None
    }

    pub async fn list(&self) -> Vec<BotStatus> {
        let bots = self.bots.read().await;
        let mut result = vec![];
        for bot in bots.values() {
            let instance = bot.lock().await;
            let mut status = instance.status.clone();
            let guard = instance.handle.lock().await;
            if let Some(ref h) = *guard {
                if !h.is_alive() && status.state == BotState::Running {
                    status.state = BotState::Error;
                }
            }
            result.push(status);
        }
        result
    }

    pub async fn get(&self, id: Uuid) -> Option<BotStatus> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)?;
        let instance = bot.lock().await;
        let mut status = instance.status.clone();
        let guard = instance.handle.lock().await;
        if let Some(ref h) = *guard {
            if !h.is_alive() && status.state == BotState::Running {
                status.state = BotState::Error;
            }
        }
        Some(status)
    }

    /// Send a message and return a broadcast receiver for streaming events
    pub async fn send_message(&self, id: Uuid, content: &str) -> Result<broadcast::Receiver<AgentEvent>> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Bot not found"))?;

        let mut instance = bot.lock().await;

        // Auto-resume if Suspended
        if instance.status.state == BotState::Suspended {
            info!("Bot {} is suspended, auto-resuming...", id);
            let engine_key = Self::engine_key(&instance.config.engine);
            let engine = self.engines.get(engine_key).unwrap().clone();

            let sid = instance.session_id.read().await.clone();
            match engine.spawn_with_options(
                &instance.config,
                sid.as_deref(),
                instance.permission_mode.as_ref(),
            ).await {
                Ok(new_handle) => {
                    {
                        let mut guard = instance.handle.lock().await;
                        *guard = Some(new_handle);
                    }
                    instance.status.state = BotState::Running;
                    info!("Bot {} resumed from suspended state", id);
                }
                Err(e) => {
                    error!("Failed to resume bot {}: {}", id, e);
                    return Err(anyhow::anyhow!("Failed to resume: {}", e));
                }
            }
        }

        info!("send_message: bot={}, engine={:?}", id, instance.config.engine);

        let engine_key = Self::engine_key(&instance.config.engine);
        let engine = self.engines.get(engine_key).unwrap();

        let handle_guard = instance.handle.lock().await;
        if let Some(ref handle) = *handle_guard {
            let rx = engine.subscribe(handle);
            info!("send_message: sending to stdin...");
            engine.send(handle, content).await?;
            info!("send_message: sent successfully");
            drop(handle_guard);
            instance.status.message_count += 1;

            // Update last_active_at in persisted record
            let current_session_id = instance.session_id.read().await.clone();
            let record = BotRecord {
                config: instance.config.clone(),
                status: instance.status.clone(),
                session_id: current_session_id,
                permission_mode: instance.permission_mode.clone(),
                idle_timeout_mins: instance.idle_timeout_mins,
                created_at: instance.status.created_at,
                last_active_at: Utc::now(),
                message_count: instance.status.message_count,
                restart_count: 0,
            };
            let _ = self.session_store.save_bot(&record).await;

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

        let handle_guard = instance.handle.lock().await;
        if let Some(ref handle) = *handle_guard {
            handle.send_line(text).await?;
            drop(handle_guard);
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
        let handle_guard = instance.handle.lock().await;

        if let Some(ref handle) = *handle_guard {
            Ok(handle.subscribe())
        } else {
            Err(anyhow::anyhow!("Bot process not running"))
        }
    }

    /// Update the cached session_id for a bot (called when we see init event)
    pub async fn set_session_id(&self, id: Uuid, session_id: String) {
        let bots = self.bots.read().await;
        if let Some(bot) = bots.get(&id) {
            let instance = bot.lock().await;
            // Update the shared Arc so supervisor can use it on restart
            *instance.session_id.write().await = Some(session_id.clone());
            // Persist
            let record = BotRecord {
                config: instance.config.clone(),
                status: instance.status.clone(),
                session_id: Some(session_id),
                permission_mode: instance.permission_mode.clone(),
                idle_timeout_mins: instance.idle_timeout_mins,
                created_at: instance.status.created_at,
                last_active_at: Utc::now(),
                message_count: instance.status.message_count,
                restart_count: 0,
            };
            let _ = self.session_store.save_bot(&record).await;
        }
    }

    /// Suspend a bot (stop process, keep config for auto-resume)
    pub async fn suspend(&self, id: Uuid) -> Result<()> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Bot not found"))?;

        let mut instance = bot.lock().await;

        // Cancel supervisor
        if let Some(supervisor) = instance.supervisor_handle.take() {
            supervisor.abort();
        }

        // Stop process
        {
            let mut handle_guard = instance.handle.lock().await;
            if let Some(ref handle) = *handle_guard {
                let _ = handle.stop().await;
            }
            *handle_guard = None;
        }

        instance.status.state = BotState::Suspended;
        info!("Bot suspended: {} ({})", instance.status.name, id);

        // Persist suspended state
        let current_session_id = instance.session_id.read().await.clone();
        let record = BotRecord {
            config: instance.config.clone(),
            status: instance.status.clone(),
            session_id: current_session_id,
            permission_mode: instance.permission_mode.clone(),
            idle_timeout_mins: instance.idle_timeout_mins,
            created_at: instance.status.created_at,
            last_active_at: Utc::now(),
            message_count: instance.status.message_count,
            restart_count: 0,
        };
        let _ = self.session_store.save_bot(&record).await;

        Ok(())
    }

    /// Get bot's idle timeout (or default 30 mins)
    pub async fn get_idle_timeout(&self, id: Uuid) -> Option<u32> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)?;
        let instance = bot.lock().await;
        Some(instance.idle_timeout_mins.unwrap_or(30))
    }

    /// Get all bot IDs and their last_active_at for idle checking
    pub async fn get_active_bots(&self) -> Vec<(Uuid, BotState)> {
        let bots = self.bots.read().await;
        let mut result = Vec::new();
        for (id, bot) in bots.iter() {
            let instance = bot.lock().await;
            result.push((*id, instance.status.state.clone()));
        }
        result
    }

    pub async fn stop(&self, id: Uuid) -> Result<()> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Bot not found"))?;

        let mut instance = bot.lock().await;

        // Cancel supervisor first
        if let Some(supervisor) = instance.supervisor_handle.take() {
            supervisor.abort();
            info!("Supervisor cancelled for bot {}", id);
        }

        // Stop the process
        {
            let mut handle_guard = instance.handle.lock().await;
            if let Some(ref handle) = *handle_guard {
                handle.stop().await?;
            }
            *handle_guard = None;
        }

        instance.status.state = BotState::Stopped;
        info!("Bot stopped: {} ({})", instance.status.name, id);

        // Persist stopped state
        let current_session_id = instance.session_id.read().await.clone();
        let record = BotRecord {
            config: instance.config.clone(),
            status: instance.status.clone(),
            session_id: current_session_id,
            permission_mode: instance.permission_mode.clone(),
            idle_timeout_mins: instance.idle_timeout_mins,
            created_at: instance.status.created_at,
            last_active_at: Utc::now(),
            message_count: instance.status.message_count,
            restart_count: 0,
        };
        let _ = self.session_store.save_bot(&record).await;

        Ok(())
    }

    /// Append event to session log and event buffer
    pub async fn log_event(&self, bot_id: Uuid, event: &AgentEvent) {
        let _ = self.session_store.append_event(bot_id, event).await;

        // Also buffer for WebSocket replay
        let bots = self.bots.read().await;
        if let Some(bot) = bots.get(&bot_id) {
            let mut instance = bot.lock().await;
            instance.event_buffer.push(event.clone());
            instance.last_active_at = Utc::now();
        }
    }

    /// Get buffered events for replay on new WebSocket connection
    pub async fn get_buffered_events(&self, id: Uuid) -> Vec<AgentEvent> {
        let bots = self.bots.read().await;
        if let Some(bot) = bots.get(&id) {
            let instance = bot.lock().await;
            instance.event_buffer.replay()
        } else {
            Vec::new()
        }
    }

    /// Get how long a bot has been idle
    pub async fn get_idle_duration(&self, id: Uuid) -> Option<Duration> {
        let bots = self.bots.read().await;
        let bot = bots.get(&id)?;
        let instance = bot.lock().await;
        let now = Utc::now();
        let elapsed = now.signed_duration_since(instance.last_active_at);
        Some(elapsed.to_std().unwrap_or(Duration::ZERO))
    }

    /// Delete a bot permanently — stop process, remove from memory AND disk
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        // First stop cleanly (cancels supervisor, kills process)
        self.stop(id).await?;

        // Remove from in-memory map
        self.bots.write().await.remove(&id);

        // Remove from disk
        self.session_store.delete_bot(id).await?;
        info!("Bot deleted: {}", id);

        Ok(())
    }
}
