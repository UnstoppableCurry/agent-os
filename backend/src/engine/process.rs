use std::process::ExitStatus;
use std::sync::Arc;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{broadcast, watch, Mutex};
use tracing::{debug, error, info};

use crate::types::AgentEvent;

/// Handle to a running CLI process
pub struct ProcessHandle {
    pub pid: u32,
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    event_tx: broadcast::Sender<AgentEvent>,
    exit_rx: watch::Receiver<Option<ExitStatus>>,
}

impl ProcessHandle {
    /// Spawn a CLI process with the given command and args
    pub async fn spawn(cmd: &str, args: &[&str], env: &[(&str, &str)], working_dir: Option<&str>) -> Result<Self> {
        let mut command = Command::new(cmd);
        command
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        for (k, v) in env {
            command.env(k, v);
        }

        // Remove all Claude-related env vars to prevent nesting detection
        command.env_remove("CLAUDECODE");
        command.env_remove("CLAUDE_CODE_ENTRYPOINT");
        command.env_remove("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS");

        let mut child = command.spawn()?;
        let pid = child.id().unwrap_or(0);

        let stdin = child.stdin.take().expect("stdin not captured");
        let stdout = child.stdout.take().expect("stdout not captured");
        let stderr = child.stderr.take().expect("stderr not captured");

        // broadcast channel: 1024 buffer
        let (event_tx, _) = broadcast::channel::<AgentEvent>(1024);
        let tx = event_tx.clone();
        let tx2 = event_tx.clone();

        // Exit status notification channel
        let (exit_tx, exit_rx) = watch::channel::<Option<ExitStatus>>(None);

        // Stdout reader — parse NDJSON, fallback to raw text
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                // Try JSON parse first (for stream-json mode)
                let event = match serde_json::from_str::<AgentEvent>(&line) {
                    Ok(event) => {
                        debug!("Process {} event: {:?}", pid, event);
                        event
                    }
                    Err(_) => {
                        // Fallback: raw text (strip ANSI codes)
                        let clean = strip_ansi(&line);
                        if clean.trim().is_empty() {
                            continue;
                        }
                        debug!("Process {} raw: {}", pid, clean);
                        AgentEvent::Raw { text: clean }
                    }
                };
                let subscribers = tx.receiver_count();
                let sent = tx.send(event);
                debug!("Process {} broadcast: subscribers={}, sent={:?}", pid, subscribers, sent.is_ok());
            }
            info!("Process {} stdout reader exited", pid);
        });

        // Stderr reader — send as raw text
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let clean = strip_ansi(&line);
                if clean.trim().is_empty() {
                    continue;
                }
                let _ = tx2.send(AgentEvent::Raw { text: clean.clone() });
                debug!("Process {} stderr: {}", pid, clean);
            }
        });

        // Process exit watcher — notifies via watch channel
        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    info!("Process {} exited with {}", pid, status);
                    let _ = exit_tx.send(Some(status));
                }
                Err(e) => {
                    error!("Process {} wait error: {}", pid, e);
                    let _ = exit_tx.send(None);
                }
            }
        });

        Ok(Self {
            pid,
            stdin: Arc::new(Mutex::new(stdin)),
            event_tx,
            exit_rx,
        })
    }

    /// Send a line to stdin
    pub async fn send_line(&self, line: &str) -> Result<()> {
        debug!("send_line pid={}: {}", self.pid, &line[..line.len().min(200)]);
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(line.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        debug!("send_line pid={}: flushed", self.pid);
        Ok(())
    }

    /// Subscribe to events (can be called multiple times)
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_tx.subscribe()
    }

    /// Get a receiver for process exit notification
    pub fn exit_receiver(&self) -> watch::Receiver<Option<ExitStatus>> {
        self.exit_rx.clone()
    }

    /// Check if the process is still alive
    pub fn is_alive(&self) -> bool {
        self.exit_rx.borrow().is_none()
    }

    /// Stop the process
    pub async fn stop(&self) -> Result<()> {
        let mut stdin = self.stdin.lock().await;
        stdin.shutdown().await?;
        Ok(())
    }
}

/// Strip ANSI escape codes from text
fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(ch) = chars.next() {
                        if ch == '\x07' {
                            break;
                        }
                        if ch == '\x1b' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                                break;
                            }
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
        } else if c == '\r' {
            continue;
        } else {
            result.push(c);
        }
    }
    result
}
