use std::sync::Arc;
use std::time::Duration;

use tokio::time;
use tracing::warn;

use crate::bot::BotManager;
use crate::types::BotState;

/// Periodically checks for idle bots and suspends them.
pub struct IdleManager {
    default_timeout: Duration,
}

impl IdleManager {
    pub fn new(default_timeout_mins: u32) -> Self {
        Self {
            default_timeout: Duration::from_secs(default_timeout_mins as u64 * 60),
        }
    }

    /// Start the idle check loop. Runs every 60 seconds.
    pub async fn run(self, manager: Arc<BotManager>) {
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;

            let bots = manager.get_active_bots().await;
            for (id, state) in bots {
                if state != BotState::Running {
                    continue;
                }

                let timeout_mins = manager.get_idle_timeout(id).await.unwrap_or(30);
                let timeout = Duration::from_secs(timeout_mins as u64 * 60);

                if let Some(idle_duration) = manager.get_idle_duration(id).await {
                    if idle_duration > timeout {
                        warn!("Bot {} idle for {:?}, suspending", id, idle_duration);
                        if let Err(e) = manager.suspend(id).await {
                            tracing::error!("Failed to suspend bot {}: {}", id, e);
                        }
                    }
                }
            }
        }
    }
}
