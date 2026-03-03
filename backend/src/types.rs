use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Stream Event (统一事件格式) ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub ts: DateTime<Utc>,
    pub source: String,      // "chat:claude", "app:dental-exam", "sensor:healthkit"
    #[serde(rename = "type")]
    pub event_type: String,   // "message", "study_session", "health_summary"
    pub data: serde_json::Value,
    #[serde(default)]
    pub meta: serde_json::Value,
}

// ─── Bot 配置 ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub id: Uuid,
    pub name: String,
    pub engine: EngineType,
    pub role: BotRole,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EngineType {
    Claude,
    Kimi,
    Codex,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BotRole {
    Boss,
    Worker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStatus {
    pub id: Uuid,
    pub name: String,
    pub engine: EngineType,
    pub role: BotRole,
    pub state: BotState,
    pub created_at: DateTime<Utc>,
    pub message_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BotState {
    Running,
    Stopped,
    Error,
    Starting,
}

// ─── Agent Engine Events (CLI 进程输出) ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    #[serde(rename = "raw")]
    Raw { text: String },
    #[serde(rename = "message_start")]
    MessageStart { message: serde_json::Value },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: u32, content_block: serde_json::Value },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: u32, delta: serde_json::Value },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },
    #[serde(rename = "message_stop")]
    MessageStop { stop_reason: Option<String> },
    #[serde(rename = "result")]
    Result { result: serde_json::Value, subtype: Option<String> },
    #[serde(other)]
    Unknown,
}

// ─── Engine Capabilities ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    pub name: String,
    pub supports_streaming: bool,
    pub supports_tools: bool,
}

// ─── API Request/Response ───

#[derive(Debug, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub engine: EngineType,
    #[serde(default = "default_worker_role")]
    pub role: BotRole,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
}

fn default_worker_role() -> BotRole {
    BotRole::Worker
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, data: None, error: Some(msg.into()) }
    }
}

// ─── Sensor Events (来自 App/传感器) ───

#[derive(Debug, Deserialize)]
pub struct IngestEventsRequest {
    pub app_id: String,
    #[serde(default)]
    pub device_id: Option<String>,
    pub events: Vec<RawEvent>,
}

#[derive(Debug, Deserialize)]
pub struct RawEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub ts: Option<DateTime<Utc>>,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub meta: serde_json::Value,
}
