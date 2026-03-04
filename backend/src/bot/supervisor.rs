use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::engine::{AgentEngine, ProcessHandle};
use crate::types::{BotConfig, PermissionMode};

/// Monitors a bot's process and auto-restarts on crash with exponential backoff.
pub struct ProcessSupervisor {
    bot_id: uuid::Uuid,
    restart_count: u32,
    max_restarts: u32,
    backoff_base: Duration,
    last_restart: Option<Instant>,
}

impl ProcessSupervisor {
    pub fn new(bot_id: uuid::Uuid) -> Self {
        Self {
            bot_id,
            restart_count: 0,
            max_restarts: 5,
            backoff_base: Duration::from_secs(1),
            last_restart: None,
        }
    }

    /// Compute next backoff delay: 1s, 2s, 4s, 8s, 16s
    fn next_backoff(&self) -> Duration {
        self.backoff_base * 2u32.saturating_pow(self.restart_count.saturating_sub(1).min(4))
    }

    /// Reset backoff counter (called after process runs successfully for >60s)
    fn reset_backoff(&mut self) {
        self.restart_count = 0;
        self.last_restart = None;
    }

    /// Start the supervisor loop. Returns a JoinHandle that can be cancelled.
    ///
    /// The supervisor watches for process exit and restarts it, resuming the
    /// Claude Code session if a session_id is available.
    pub fn start(
        bot_id: uuid::Uuid,
        config: BotConfig,
        initial_handle: Arc<Mutex<Option<ProcessHandle>>>,
        engine: Arc<dyn AgentEngine>,
        state_tx: tokio::sync::mpsc::Sender<SupervisorEvent>,
        session_id: Arc<RwLock<Option<String>>>,
        permission_mode: Option<PermissionMode>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut supervisor = Self::new(bot_id);
            supervisor.run_loop(config, initial_handle, engine, state_tx, session_id, permission_mode).await;
        })
    }

    async fn run_loop(
        &mut self,
        config: BotConfig,
        handle: Arc<Mutex<Option<ProcessHandle>>>,
        engine: Arc<dyn AgentEngine>,
        state_tx: tokio::sync::mpsc::Sender<SupervisorEvent>,
        session_id: Arc<RwLock<Option<String>>>,
        permission_mode: Option<PermissionMode>,
    ) {
        loop {
            // Get exit receiver from current handle
            let mut exit_rx = {
                let guard = handle.lock().await;
                match guard.as_ref() {
                    Some(h) => h.exit_receiver(),
                    None => {
                        info!("Supervisor {}: no process handle, exiting", self.bot_id);
                        return;
                    }
                }
            };

            let started_at = Instant::now();

            // Wait for process to exit
            loop {
                if exit_rx.changed().await.is_err() {
                    info!("Supervisor {}: exit channel closed, stopping", self.bot_id);
                    return;
                }
                if exit_rx.borrow().is_some() {
                    break;
                }
            }

            let exit_status = exit_rx.borrow().clone();
            let run_duration = started_at.elapsed();

            info!(
                "Supervisor {}: process exited (status={:?}, ran for {:?})",
                self.bot_id, exit_status, run_duration
            );

            // If process ran for more than 60s, reset backoff
            if run_duration > Duration::from_secs(60) {
                self.reset_backoff();
            }

            self.restart_count += 1;

            // Check if we've exceeded max restarts
            if self.restart_count > self.max_restarts {
                error!(
                    "Supervisor {}: exceeded max restarts ({}), marking as Error",
                    self.bot_id, self.max_restarts
                );
                let _ = state_tx.send(SupervisorEvent::MaxRestartsExceeded).await;
                return;
            }

            // Exponential backoff delay
            let delay = self.next_backoff();
            warn!(
                "Supervisor {}: restarting in {:?} (attempt {}/{})",
                self.bot_id, delay, self.restart_count, self.max_restarts
            );
            let _ = state_tx.send(SupervisorEvent::Restarting {
                attempt: self.restart_count,
                delay,
            }).await;

            tokio::time::sleep(delay).await;
            self.last_restart = Some(Instant::now());

            // Attempt restart — resume session if we have a session_id
            let sid = session_id.read().await.clone();
            match engine.spawn_with_options(&config, sid.as_deref(), permission_mode.as_ref()).await {
                Ok(new_handle) => {
                    info!("Supervisor {}: process restarted, pid={}", self.bot_id, new_handle.pid);
                    let mut guard = handle.lock().await;
                    *guard = Some(new_handle);
                    let _ = state_tx.send(SupervisorEvent::Restarted).await;
                }
                Err(e) => {
                    error!("Supervisor {}: failed to restart: {}", self.bot_id, e);
                    let _ = state_tx.send(SupervisorEvent::RestartFailed(e.to_string())).await;
                    // Continue loop to retry
                }
            }
        }
    }
}

/// Events emitted by the supervisor to the BotManager
#[derive(Debug)]
pub enum SupervisorEvent {
    Restarting { attempt: u32, delay: Duration },
    Restarted,
    RestartFailed(String),
    MaxRestartsExceeded,
}
