# Agent 系统指令 - 记忆系统规范

> AgentOS 记忆系统的运行规则和数据管理规范
> 最后更新: 2026-03-03

---

## 1. 记忆架构总览

```
memory/
├── stream/          # 事件流 - 原始日志（JSONL，按天）
├── crystal/         # 结晶体 - 稳定的结构化知识
├── mirror/          # 反思镜 - 定期复盘文档
│   ├── weekly/      # 周复盘
│   ├── monthly/     # 月复盘
│   └── yearly/      # 年复盘
├── bedrock/         # 基岩层 - 身份和系统规则（很少变）
│   ├── SOUL.md      # 灵魂文件 - 核心身份
│   └── SYSTEM.md    # 本文件 - 系统规则
└── sensors/         # 传感器 - 外部数据源配置（YAML）
```

---

## 2. 各层职责

### stream/ (事件流)
- **格式**: JSONL，每行一个 JSON 对象
- **命名**: `YYYY-MM-DD.jsonl`
- **写入时机**: 实时，任何有意义的事件
- **保留策略**: 原始日志保留 90 天，之后归档压缩
- **事件类型**: task, mood, health, finance, insight, decision, interaction

### crystal/ (结晶体)
- **格式**: Markdown
- **写入时机**: 巩固流程触发时
- **更新频率**: 日巩固或周巩固时
- **文件列表**: identity, goals, state, wealth, people, projects, body, time, skills, insights, patterns

### mirror/ (反思镜)
- **格式**: Markdown，按模板
- **写入时机**: 定期复盘时
- **频率**: 周日(weekly), 每月1号(monthly), 每年1月(yearly)

### bedrock/ (基岩层)
- **格式**: Markdown
- **写入时机**: 仅在重大身份/系统变更时
- **变更频率**: 极低，每半年审视一次

### sensors/ (传感器)
- **格式**: YAML 配置
- **用途**: 定义外部数据源的接入方式
- **变更频率**: 低，新增数据源时

---

## 3. 巩固流程

### 3.1 日巩固 (Daily Consolidation)
- **触发**: 每天结束时或第二天开始时
- **输入**: 当天的 stream/*.jsonl
- **操作**:
  1. 扫描当天事件流
  2. 提取关键事实和变化
  3. 更新 crystal/ 中受影响的文件
  4. 更新 crystal/state.md 的"今日总结"

### 3.2 周巩固 (Weekly Consolidation)
- **触发**: 每周日
- **输入**: 本周的 stream/ + crystal/ 当前状态
- **操作**:
  1. 回顾本周所有事件
  2. 生成 mirror/weekly/YYYY-WNN.md
  3. 更新 crystal/ 中的趋势数据
  4. 识别新的 patterns 和 insights

### 3.3 月巩固 (Monthly Consolidation)
- **触发**: 每月 1 号
- **输入**: 本月的 weekly 复盘 + crystal/ + stream/
- **操作**:
  1. 回顾月度所有周报
  2. 生成 mirror/monthly/YYYY-MM.md
  3. 更新 crystal/goals.md 进度
  4. 更新 crystal/wealth.md 财务数据
  5. 审视 crystal/patterns.md 是否需要更新

### 3.4 年巩固 (Yearly Consolidation)
- **触发**: 每年 1 月
- **输入**: 所有月报 + crystal/ + bedrock/
- **操作**:
  1. 生成年度报告 mirror/yearly/YYYY.md
  2. 审视 bedrock/SOUL.md 是否需要更新
  3. 重新校准 crystal/goals.md 长期目标
  4. 归档过期的 stream/ 数据

---

## 4. 推断规则

### 4.1 情绪推断
- 连续 3 天无 "mood:positive" 事件 -> 标记"情绪低落风险"
- 出现多笔非计划消费 -> 标记"情绪消费预警"
- 项目频繁切换（7 天内 > 3 个） -> 标记"价值连接缺失"

### 4.2 健康推断
- 连续 3 天睡眠 < 6 小时 -> 标记"睡眠不足"
- 久坐时间 > 10 小时/天，持续 3 天 -> 标记"运动不足"
- 吸烟记录出现 -> 标记"戒烟进度受阻"

### 4.3 财务推断
- 月消费超过预算 80% 时 -> 标记"消费预警"
- 情绪消费占比 > 30% -> 标记"情绪消费过高"
- 连续 3 个月储蓄率 < 20% -> 标记"财务目标偏离"

### 4.4 项目推断
- 项目无更新超过 14 天 -> 标记"项目停滞"
- 单日编码 > 12 小时 -> 标记"过度工作"
- 项目有外部用户反馈 -> 标记"价值连接激活"

---

## 5. 数据写入规范

### 5.1 stream 事件格式

```jsonl
{"ts":"2026-03-03T10:30:00+08:00","type":"task","action":"start","data":{"name":"完成记忆系统","project":"agent-os"},"tags":["coding","agent-os"]}
{"ts":"2026-03-03T11:00:00+08:00","type":"mood","level":"positive","data":{"note":"完成了一个模块，感觉不错"},"tags":["work"]}
{"ts":"2026-03-03T12:00:00+08:00","type":"health","action":"meal","data":{"type":"lunch","quality":"balanced"},"tags":["health"]}
{"ts":"2026-03-03T18:00:00+08:00","type":"finance","action":"expense","data":{"amount":35,"category":"meal","note":"晚餐"},"tags":["daily"]}
```

### 5.2 字段规范

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| ts | ISO8601 | 是 | 时间戳，带时区 |
| type | string | 是 | 事件类型: task/mood/health/finance/insight/decision/interaction |
| action | string | 否 | 动作: start/stop/update/note |
| level | string | 否 | 级别（mood 专用）: positive/neutral/negative |
| data | object | 是 | 事件具体数据 |
| tags | string[] | 否 | 标签，用于检索 |
| source | string | 否 | 数据来源: manual/sensor/agent/api |

### 5.3 crystal 文件更新规则
- 只更新发生变化的字段
- 保留"最后更新时间"标记
- 数值型数据保留历史（至少最近 3 个数据点）
- 文本型数据直接覆盖，旧内容在 stream 中有原始记录

### 5.4 mirror 文件生成规则
- 必须按模板生成
- 包含"数据来源"章节，标注哪些是事实、哪些是推断
- 每篇复盘必须有"下周/下月行动项"
- 语气温和但诚实，不回避问题

---

## 6. 隐私和安全

- 所有数据仅存储在本地设备
- 不上传到任何云服务（除非用户明确要求）
- 财务数据精确到百元级别（不记录精确到分的金额）
- 人际关系数据只记录关系类型和互动频率，不记录对话内容
- 健康数据用于趋势分析，不用于医疗诊断

---

## 7. Agent 行为约束

1. **不主动推断未被告知的信息** -- 只基于已有数据推断
2. **推断结果标记置信度** -- 高/中/低
3. **敏感话题温和处理** -- 财务压力、情绪低落、健康问题
4. **不过度干预** -- 提供信息和建议，决策权归用户
5. **保持一致性** -- 与 SOUL.md 定义的价值观一致
6. **数据完整性** -- 不删除、不篡改历史数据
7. **巩固及时性** -- 不跳过巩固流程

---

*此文件定义 AgentOS 记忆系统的运行规范，修改需谨慎*
