# Skills 系统

Skills 是 AgentOS 的扩展能力单元。每个 Skill 是一个目录，包含 `SKILL.md` 定义文件。

## 结构

```
skills/
├── README.md
└── skill-name/
    └── SKILL.md
```

## SKILL.md 格式

```yaml
---
name: skill-name
description: 一句话描述
always: true/false    # true = 始终加载, false = 按需加载
triggers:             # 触发条件（always=false 时）
  - pattern: "关键词"
---

# 详细说明和指令
```

## 内置 Skills

| Skill | 说明 | 加载方式 |
|-------|------|----------|
| `git-workflow` | Git 操作辅助 | always |
