# 项目总览

> 最后更新: 2026-03-03

---

## 活跃项目

### AgentOS
- **状态**: 开发中 (刚启动)
- **目标**: 个人 AI 操作系统 -- 记忆、任务、健康、财务一体化管理
- **本地路径**: `/Users/wangtianxin/projects/agent-os/`
- **技术栈**: Rust 后端 + SwiftUI 前端 + Swift Package (LifeKit)
- **价值连接**: 自己的 Jarvis，也可以帮助他人管理人生 (**符合梦想清单**)
- **优先级**: P0

### claude-code-api
- **状态**: 已部署运行
- **目标**: 将 Claude Code CLI API 化
- **本地路径**: `/Users/wangtianxin/Desktop/claude/claude/claude-code-api/`
- **GitHub**: https://github.com/UnstoppableCurry/claude-code-api
- **部署**: 49.232.225.189:3000
- **技术栈**: TypeScript, Fastify, WebSocket
- **优先级**: P1 (维护模式)

### VideoKnowledge
- **状态**: 待修复
- **目标**: 视频知识管理 -- CloudKit 同步 + iOS/macOS 双端
- **本地路径**: `/Users/wangtianxin/Desktop/视频知识/VideoKnowledge`
- **GitHub**: https://github.com/UnstoppableCurry/VideoKnowledge.git
- **待修复**: 链接无效 + 图片缺失 + 懒加载 + 登录页
- **优先级**: P2

---

## 已完成/存档项目

### ai-task-platform
- **本地路径**: `/Users/wangtianxin/Desktop/ai-task-platform`
- **GitHub**: https://github.com/UnstoppableCurry/ai-task-platform
- **备注**: AI 任务平台，22次提交

### health-sync-platform
- **本地路径**: `/Users/wangtianxin/Desktop/health-sync-platform`
- **GitHub**: https://github.com/UnstoppableCurry/health-sync-platform
- **备注**: 健康数据同步平台，7次提交

### DentalExamPrep (牙医资格证题库)
- **本地路径**: `/Users/wangtianxin/Desktop/牙医资格证题库`
- **GitHub**: https://github.com/UnstoppableCurry/DentalExamPrep-iOS
- **备注**: iOS 牙医题库 App, 1822 题

---

## 已弃坑项目 (教训保留)

### cc-api-cloud
- **方案**: CLI spawn 包装
- **弃坑原因**: 不稳定，进程管理复杂，延迟高
- **教训**: 不要用 spawn/PTY 包装 CLI

### cc-api-system
- **方案**: 直接调用 Anthropic API 重写
- **弃坑原因**: 丢失 Claude Code 全部工具和 prompt 优化
- **教训**: 不要丢掉已有的核心竞争力去重写

---

## 项目选择原则 (基于 SOUL)

> 启动新项目前回答: "做成了，谁会因此受益？我能看见那个受益的人吗？"

1. 必须有明确的价值连接点
2. 优先解决自己的真实痛点
3. 能独立完成、能展示、能收到反馈
4. 控制同时进行的项目数 <= 3
