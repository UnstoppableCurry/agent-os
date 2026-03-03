use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::types::StreamEvent;

/// Append-only event stream backed by daily JSONL files
pub struct StreamStore {
    base_dir: PathBuf,
}

impl StreamStore {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Append an event to today's stream file
    pub async fn append(&self, event: &StreamEvent) -> Result<()> {
        let date = event.ts.format("%Y-%m-%d").to_string();
        let file_path = self.base_dir.join(format!("{}.jsonl", date));

        fs::create_dir_all(&self.base_dir).await?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .await?;

        let line = serde_json::to_string(event)?;
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;

        Ok(())
    }

    /// Append a raw event from a sensor/app
    pub async fn append_raw(
        &self,
        source: &str,
        event_type: &str,
        data: serde_json::Value,
        meta: serde_json::Value,
    ) -> Result<()> {
        let event = StreamEvent {
            ts: Utc::now(),
            source: source.to_string(),
            event_type: event_type.to_string(),
            data,
            meta,
        };
        self.append(&event).await
    }

    /// Read all events from a specific date
    pub async fn read_day(&self, date: &str) -> Result<Vec<StreamEvent>> {
        let file_path = self.base_dir.join(format!("{}.jsonl", date));

        if !file_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&file_path).await?;
        let events: Vec<StreamEvent> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        Ok(events)
    }

    /// Read today's events
    pub async fn read_today(&self) -> Result<Vec<StreamEvent>> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        self.read_day(&today).await
    }

    /// List available stream dates
    pub async fn list_dates(&self) -> Result<Vec<String>> {
        let mut dates = vec![];
        let mut entries = fs::read_dir(&self.base_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".jsonl") {
                dates.push(name.trim_end_matches(".jsonl").to_string());
            }
        }

        dates.sort();
        Ok(dates)
    }
}
