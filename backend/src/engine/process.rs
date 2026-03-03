use std::sync::Arc;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info, warn};

use crate::types::AgentEvent;

/// Handle to a running CLI process
pub struct ProcessHandle {
    pub pid: u32,
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    event_tx: broadcast::Sender<AgentEvent>,
}

impl ProcessHandle {
    /// Spawn a CLI process with the given command and args
    pub async fn spawn(cmd: &str, args: &[&str], env: &[(&str, &str)]) -> Result<Self> {
        let mut command = Command::new(cmd);
        command
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        for (k, v) in env {
            command.env(k, v);
        }

        // Remove CLAUDECODE env to bypass nested session detection
        command.env_remove("CLAUDECODE");

        let mut child = command.spawn()?;
        let pid = child.id().unwrap_or(0);

        let stdin = child.stdin.take().expect("stdin not captured");
        let stdout = child.stdout.take().expect("stdout not captured");
        let stderr = child.stderr.take().expect("stderr not captured");

        // broadcast channel: 256 buffer, multiple subscribers
        let (event_tx, _) = broadcast::channel::<AgentEvent>(256);
        let tx = event_tx.clone();

        // Stdout reader — parse NDJSON events
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                let event = match serde_json::from_str::<AgentEvent>(&line) {
                    Ok(event) => event,
                    Err(_) => AgentEvent::Result {
                        result: serde_json::json!({"text": line}),
                        subtype: Some("raw".to_string()),
                    },
                };
                // If no subscribers, the send will fail — that's fine
                let _ = tx.send(event);
            }
            info!("Process {} stdout reader exited", pid);
        });

        // Stderr reader — log warnings
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.trim().is_empty() {
                    warn!("Process {} stderr: {}", pid, line);
                }
            }
        });

        // Process exit watcher
        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => info!("Process {} exited with {}", pid, status),
                Err(e) => error!("Process {} wait error: {}", pid, e),
            }
        });

        Ok(Self {
            pid,
            stdin: Arc::new(Mutex::new(stdin)),
            event_tx,
        })
    }

    /// Send a line to stdin
    pub async fn send_line(&self, line: &str) -> Result<()> {
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(line.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }

    /// Subscribe to events (can be called multiple times)
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_tx.subscribe()
    }

    /// Stop the process
    pub async fn stop(&self) -> Result<()> {
        // Drop the stdin to signal EOF to the child
        let mut stdin = self.stdin.lock().await;
        stdin.shutdown().await?;
        Ok(())
    }
}
