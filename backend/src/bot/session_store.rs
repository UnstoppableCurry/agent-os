use std::path::PathBuf;

use anyhow::Result;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};
use uuid::Uuid;

use crate::types::{AgentEvent, BotRecord};

/// File-based session persistence.
///
/// Storage layout:
///   data/bots/{uuid}.json      — Bot config + state
///   data/sessions/{uuid}.jsonl — Event log
pub struct SessionStore {
    data_dir: PathBuf,
}

impl SessionStore {
    pub async fn new(data_dir: impl Into<PathBuf>) -> Result<Self> {
        let data_dir = data_dir.into();
        fs::create_dir_all(data_dir.join("bots")).await?;
        fs::create_dir_all(data_dir.join("sessions")).await?;
        info!("SessionStore initialized at {:?}", data_dir);
        Ok(Self { data_dir })
    }

    fn bot_path(&self, id: Uuid) -> PathBuf {
        self.data_dir.join("bots").join(format!("{}.json", id))
    }

    fn session_path(&self, id: Uuid) -> PathBuf {
        self.data_dir.join("sessions").join(format!("{}.jsonl", id))
    }

    /// Save bot record to disk
    pub async fn save_bot(&self, record: &BotRecord) -> Result<()> {
        let path = self.bot_path(record.config.id);
        let json = serde_json::to_string_pretty(record)?;
        fs::write(&path, json).await?;
        debug!("Saved bot record: {:?}", path);
        Ok(())
    }

    /// Load a single bot record
    pub async fn load_bot(&self, id: Uuid) -> Result<Option<BotRecord>> {
        let path = self.bot_path(id);
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&path).await?;
        let record: BotRecord = serde_json::from_str(&data)?;
        Ok(Some(record))
    }

    /// List all saved bot records
    pub async fn list_bots(&self) -> Result<Vec<BotRecord>> {
        let bots_dir = self.data_dir.join("bots");
        let mut records = Vec::new();
        let mut entries = fs::read_dir(&bots_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                match fs::read_to_string(&path).await {
                    Ok(data) => match serde_json::from_str::<BotRecord>(&data) {
                        Ok(record) => records.push(record),
                        Err(e) => {
                            tracing::warn!("Failed to parse bot record {:?}: {}", path, e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to read bot file {:?}: {}", path, e);
                    }
                }
            }
        }
        Ok(records)
    }

    /// Delete a bot record from disk
    pub async fn delete_bot(&self, id: Uuid) -> Result<()> {
        let path = self.bot_path(id);
        if path.exists() {
            fs::remove_file(&path).await?;
        }
        // Also clean up session log
        let session_path = self.session_path(id);
        if session_path.exists() {
            fs::remove_file(&session_path).await?;
        }
        Ok(())
    }

    /// Append an event to the bot's session log
    pub async fn append_event(&self, bot_id: Uuid, event: &AgentEvent) -> Result<()> {
        let path = self.session_path(bot_id);
        let mut line = serde_json::to_string(event)?;
        line.push('\n');
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;
        file.write_all(line.as_bytes()).await?;
        Ok(())
    }

    /// Load recent events from session log (last N lines)
    pub async fn load_events(&self, bot_id: Uuid, limit: usize) -> Result<Vec<AgentEvent>> {
        let path = self.session_path(bot_id);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&path).await?;
        let events: Vec<AgentEvent> = data
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        // Return last `limit` events
        let skip = events.len().saturating_sub(limit);
        Ok(events.into_iter().skip(skip).collect())
    }
}
