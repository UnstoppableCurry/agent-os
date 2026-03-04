use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Stream Event (统一事件格式) ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub ts: DateTime<Utc>,
    pub source: String,
    #[serde(rename = "type")]
    pub event_type: String,
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
    Suspended,
}

// ─── Agent Engine Events (Claude Code stream-json 输出) ───
//
// Claude Code 的 --output-format stream-json 实际格式:
//   {"type":"system","subtype":"init","cwd":"...","session_id":"...","tools":[...]}
//   {"type":"system","subtype":"hook_started","hook_name":"..."}
//   {"type":"system","subtype":"hook_response","hook_name":"...","exit_code":0}
//   {"type":"assistant","message":{"content":[{"type":"text","text":"Hello!"}],...}}
//   {"type":"result","subtype":"success","result":"Hello!","duration_ms":1234}
//
// 注意: 这不是 Anthropic API 的 message_start/content_block_delta 格式!

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    /// 原始文本行（非JSON输出或stderr）
    #[serde(rename = "raw")]
    Raw { text: String },

    /// 系统事件: init, hook_started, hook_response
    #[serde(rename = "system")]
    System {
        subtype: Option<String>,
        session_id: Option<String>,
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        model: Option<String>,
        #[serde(default)]
        tools: Option<Vec<String>>,
        #[serde(default)]
        hook_name: Option<String>,
        #[serde(default)]
        exit_code: Option<i32>,
    },

    /// 助手回复: 包含完整 message 对象
    #[serde(rename = "assistant")]
    Assistant {
        message: serde_json::Value,
        session_id: Option<String>,
    },

    /// 用户消息回显
    #[serde(rename = "user")]
    User {
        message: Option<serde_json::Value>,
        session_id: Option<String>,
    },

    /// 最终结果
    #[serde(rename = "result")]
    Result {
        subtype: Option<String>,
        result: Option<String>,
        #[serde(default)]
        is_error: Option<bool>,
        #[serde(default)]
        duration_ms: Option<u64>,
        session_id: Option<String>,
    },

    /// SDK 控制响应 (init response, permission prompts, etc.)
    #[serde(rename = "control_response")]
    ControlResponse {
        #[serde(default)]
        response: Option<serde_json::Value>,
    },

    /// 未知事件类型
    #[serde(other)]
    Unknown,
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
    #[serde(default)]
    pub permission_mode: Option<PermissionMode>,
    #[serde(default)]
    pub resume_session: Option<String>,
    #[serde(default)]
    pub idle_timeout_mins: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    BypassPermissions,
    Default,
    AcceptEdits,
    Plan,
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

// ─── Bot Persistence Record ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotRecord {
    pub config: BotConfig,
    pub status: BotStatus,
    pub session_id: Option<String>,
    pub permission_mode: Option<PermissionMode>,
    pub idle_timeout_mins: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub message_count: u64,
    pub restart_count: u32,
}
