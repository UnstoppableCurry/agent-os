---
name: git-workflow
description: Git 操作辅助，自动化常见 Git 工作流
always: true
---

# Git Workflow Skill

## 能力

- 自动生成 conventional commit 消息
- 分支管理（创建、切换、合并）
- PR 创建和描述生成
- 冲突检测和解决建议

## Commit 格式

```
<type>(<scope>): <description>

[body]

[footer]
```

类型：
- `feat`: 新功能
- `fix`: Bug 修复
- `refactor`: 重构
- `docs`: 文档
- `chore`: 杂项
- `test`: 测试

## 分支命名

- `feat/<描述>` - 功能分支
- `fix/<描述>` - 修复分支
- `refactor/<描述>` - 重构分支
