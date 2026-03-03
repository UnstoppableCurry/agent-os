use std::sync::Arc;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, warn};

use crate::types::AgentEvent;

/// Handle to a running CLI process
pub struct ProcessHandle {
    pub pid: u32,
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    event_tx: mpsc::Sender<AgentEvent>,
    event_rx: Arc<Mutex<Option<mpsc::Receiver<AgentEvent>>>>,
    kill_tx: mpsc::Sender<()>,
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

        let mut child = command.spawn()?;
        let pid = child.id().unwrap_or(0);

        let stdin = child.stdin.take().expect("stdin not captured");
        let stdout = child.stdout.take().expect("stdout not captured");
        let stderr = child.stderr.take().expect("stderr not captured");

        let (event_tx, event_rx) = mpsc::channel::<AgentEvent>(256);
        let (kill_tx, mut kill_rx) = mpsc::channel::<()>(1);

        let tx = event_tx.clone();

        // Stdout reader - parse NDJSON events
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            loop {
                tokio::select! {
                    line = lines.next_line() => {
                        match line {
                            Ok(Some(line)) => {
                                if line.trim().is_empty() {
                                    continue;
                                }
                                match serde_json::from_str::<AgentEvent>(&line) {
                                    Ok(event) => {
                                        if tx.send(event).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(_) => {
                                        // Non-JSON line, try wrapping as result text
                                        let event = AgentEvent::Result {
                                            result: serde_json::json!({"text": line}),
                                            subtype: Some("raw".to_string()),
                                        };
                                        let _ = tx.send(event).await;
                                    }
                                }
                            }
                            Ok(None) => break, // EOF
                            Err(e) => {
                                error!("stdout read error: {}", e);
                                break;
                            }
                        }
                    }
                    _ = kill_rx.recv() => {
                        break;
                    }
                }
            }
            info!("Process {} stdout reader exited", pid);
        });

        // Stderr reader - log warnings
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
            event_rx: Arc::new(Mutex::new(Some(event_rx))),
            kill_tx,
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

    /// Take the event receiver (can only be called once)
    pub fn take_event_rx(&self) -> Option<mpsc::Receiver<AgentEvent>> {
        self.event_rx.try_lock().ok()?.take()
    }

    /// Stop the process
    pub async fn stop(&self) -> Result<()> {
        let _ = self.kill_tx.send(()).await;
        Ok(())
    }
}
