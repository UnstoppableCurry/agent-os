# AgentOS -- 个人 AI 操作系统

通过手机聊天界面管理多个 AI Agent，实现 24 小时自动化工作。
每个你开发的 App 都是 AI 的一个触角。

## 架构

- **Rust 后端**: 进程管理 + 消息路由 + 记忆引擎
- **SwiftUI App**: 聊天界面 + 数据传感器
- **LifeKit SDK**: 连接所有 App 到记忆系统

## 记忆系统：流 + 晶体

- **流 (Stream)**: 所有事件按时间序列记录，只增不改
- **晶体 (Crystal)**: Agent 从流中提炼的结构化知识
- **镜像 (Mirror)**: 定期复盘报告
- **根基 (Bedrock)**: 不变的灵魂定义

## 模块

| 目录 | 说明 |
|------|------|
| `backend/` | Rust 后端 (Axum + Tokio) |
| `app/` | SwiftUI 主 App (iOS + macOS) |
| `lifekit/` | LifeKit Swift Package (SDK) |
| `memory/` | 记忆文件系统 |
| `skills/` | Skills 扩展库 |

## 技术栈

| 层 | 技术 |
|----|------|
| 后端 | Rust, Axum, Tokio, SQLite |
| 前端 | SwiftUI, Combine |
| SDK | Swift Package Manager |
| 通信 | WebSocket + JSON |
| AI | Claude API (Anthropic) |

## 快速开始

```bash
# 构建后端
make build-backend

# 运行后端
make run-backend

# 构建 LifeKit SDK
make build-lifekit

# 运行测试
make test
```

## 开发

详见 [CLAUDE.md](./CLAUDE.md) 获取完整的架构设计和开发规范。
