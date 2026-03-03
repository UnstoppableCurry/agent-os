# AgentOS - 项目知识库

## 1. 项目目标

**构建个人 AI 操作系统**：通过手机聊天界面管理多个 AI Agent，实现 24 小时自动化。

核心理念：
- 每个 App 都是 AI 的触角，收集数据、执行任务
- 记忆系统是灵魂，所有数据汇入统一的记忆流
- Agent 是大脑，从记忆中学习、决策、行动

## 2. 架构决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 后端语言 | Rust | 性能、安全、长期运行稳定 |
| Web 框架 | Axum + Tokio | Rust 生态最成熟的异步 Web 框架 |
| 前端 | SwiftUI | Apple 全平台，iOS + macOS 统一 |
| SDK | Swift Package | 便于集成到所有 Swift App |
| 通信协议 | WebSocket | 实时双向，适合聊天和事件推送 |
| 数据库 | SQLite | 轻量、嵌入式、够用 |
| AI 接口 | Claude API | 最强推理能力 |

## 3. 模块结构

```
agent-os/
├── backend/          # Rust 后端
│   ├── src/
│   │   ├── main.rs           # 入口
│   │   ├── router.rs         # 路由
│   │   ├── ws.rs             # WebSocket 处理
│   │   ├── agent/            # Agent 管理
│   │   │   ├── mod.rs
│   │   │   ├── process.rs    # 进程生命周期
│   │   │   └── router.rs     # 消息路由
│   │   ├── memory/           # 记忆引擎
│   │   │   ├── mod.rs
│   │   │   ├── stream.rs     # 事件流（只增）
│   │   │   ├── crystal.rs    # 晶体（结构化知识）
│   │   │   └── query.rs      # 记忆检索
│   │   └── skill/            # Skills 系统
│   │       ├── mod.rs
│   │       └── loader.rs     # Skill 加载器
│   └── Cargo.toml
├── app/              # SwiftUI 主 App
│   ├── Sources/AgentOS/
│   │   ├── AgentOSApp.swift  # App 入口
│   │   ├── ContentView.swift # 主界面
│   │   ├── Views/            # 视图组件
│   │   ├── Models/           # 数据模型
│   │   ├── Network/          # 网络层
│   │   └── Sensors/          # 传感器
│   └── Package.swift
├── lifekit/          # LifeKit SDK
│   ├── Sources/LifeKit/
│   │   ├── Models.swift      # 事件模型
│   │   ├── Transport.swift   # 传输层
│   │   ├── EventBuffer.swift # 事件缓冲
│   │   └── Privacy.swift     # 隐私标注
│   ├── Tests/
│   └── Package.swift
├── memory/           # 记忆文件系统
│   ├── stream/       # 事件流（时间序列）
│   ├── crystal/      # 晶体（结构化知识）
│   ├── mirror/       # 复盘报告
│   │   ├── weekly/
│   │   ├── monthly/
│   │   └── yearly/
│   ├── sensors/      # 传感器原始数据
│   └── bedrock/      # 灵魂定义
│       └── SOUL.md
├── skills/           # Skills 扩展
└── Makefile
```

## 4. 记忆系统哲学

### 流 (Stream) - 生命的时间线
所有事件按时间戳顺序追加写入，永不修改、永不删除。
这是最原始、最真实的记录。

### 晶体 (Crystal) - 提炼的智慧
Agent 定期从流中提炼结构化知识：
- 习惯模式（几点起床、常去哪里）
- 偏好（喜欢什么、不喜欢什么）
- 关系图谱（认识谁、和谁互动多）
- 技能图谱（会什么、在学什么）

### 镜像 (Mirror) - 定期复盘
- 周报：本周做了什么，学到什么
- 月报：本月趋势，目标进展
- 年报：年度总结，成长轨迹

### 根基 (Bedrock) - 不变的灵魂
`SOUL.md` 定义了 AI 的核心人格、价值观和行为准则。
这是唯一手动编写、不由 AI 自动修改的部分。

## 5. 事件 Schema

```json
{
  "id": "uuid",
  "timestamp": "2026-03-03T12:00:00Z",
  "source": "app.location | app.health | agent.chat | skill.git",
  "type": "sensor | action | thought | decision",
  "privacy": "public | private | secret",
  "payload": {},
  "metadata": {
    "device": "iPhone",
    "app_version": "1.0.0"
  }
}
```

## 6. API 设计

### WebSocket `/ws`
双向实时通信，用于聊天和事件推送。

消息格式：
```json
{"type": "chat", "content": "你好", "bot_id": "main"}
{"type": "event", "source": "sensor.location", "payload": {...}}
{"type": "command", "action": "agent.start", "target": "researcher"}
```

### REST API

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 健康检查 |
| GET | `/api/bots` | Bot 列表 |
| GET | `/api/bots/:id/history` | 聊天历史 |
| POST | `/api/events` | 批量提交事件 |
| GET | `/api/memory/query` | 记忆检索 |
| GET | `/api/skills` | Skill 列表 |

## 7. 开发规范

- Rust 代码使用 `cargo fmt` 格式化，`cargo clippy` 检查
- Swift 代码遵循 Swift API Design Guidelines
- 所有 API 返回 JSON，错误使用标准格式 `{"error": "message"}`
- 日志使用结构化格式（Rust: tracing, Swift: os_log）
- Git commit 使用 conventional commit 格式

## 8. 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `AGENT_OS_PORT` | 后端端口 | 3000 |
| `AGENT_OS_DB_PATH` | SQLite 路径 | `./data/agent-os.db` |
| `AGENT_OS_MEMORY_DIR` | 记忆目录 | `./memory` |
| `ANTHROPIC_API_KEY` | Claude API Key | - |
| `AGENT_OS_LOG_LEVEL` | 日志级别 | info |
