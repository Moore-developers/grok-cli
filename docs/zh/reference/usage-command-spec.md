# `usage` 命令规格

## 1. 目标

新增一个可稳定被 SKILL、脚本和人工直接使用的 `grok-cli usage` 命令，用来回答当前这条 Grok 会话本地已经消耗了多少。

这里刻意把 `usage` 设计成顶层命令，而不是塞进认证命令、`state` 或能力命令：

- 它不是认证动作
- 它不是单次任务执行
- 它也不只是状态文件展示
- 它横跨本地 session 统计、最近速率限制快照和分类使用量

因此推荐命令面固定为：

```text
grok-cli usage [options]
```

## 2. 命令语义

### 2.1 首版入口

```bash
cargo run -- usage --json
```

推荐支持参数：

- `--json`
- `--auth-file <path>`
- `--session-db <path>`
- `--session-id <id>`
- `--timeout <seconds>`：隐藏兼容参数；当前不查询账号额度
- `--local-only`：隐藏兼容参数；当前默认就是本地统计

说明：

- `usage` 只输出本地 session / rate-limit 历史，不访问 provider live quota
- `--session-id` 用于显式查看某个 session；未提供时读取当前 active session
- `--session-db` 允许测试和自动化覆盖独立 SQLite 文件

### 2.2 成功与降级原则

`usage` 应优先返回本地可用结果，不依赖远程 quota 接口。

首版建议：

- 本地 session 统计成功，则整个命令返回 `ok: true`
- 不查询、不展示、不返回 Account limits
- 只有以下情况才建议整体失败：
  - 参数非法
  - SQLite schema 损坏且无法恢复
  - 本地 session store 路径不可写、不可读且无降级路径

## 3. 成功输出结构

成功信封仍沿用现有统一协议：

```json
{
  "ok": true,
  "command": "usage",
  "data": {}
}
```

`data` 建议固定为四个逻辑区块：

```json
{
  "provider": "xai-oauth",
  "session": {
    "session_id": "sess_01JV....",
    "started_at": "2026-05-20T10:00:00Z",
    "last_activity_at": "2026-05-20T10:08:34Z",
    "duration_seconds": 514,
    "request_count": 6,
    "tracked_command_count": 6,
    "models": ["grok-4.3"],
    "session_store_path": "/abs/path/session.db"
  },
  "local_usage": {
    "input_tokens": 12450,
    "output_tokens": 6912,
    "cache_read_tokens": 0,
    "cache_write_tokens": 0,
    "reasoning_tokens": 0,
    "total_tokens": 19362,
    "estimated_cost_usd": 0.184215,
    "pricing_status": "estimated",
    "pricing_source": "bundled_xai_table",
    "last_model": "grok-4.3",
    "context_window_tokens": null,
    "history_turns": 6,
    "compression_count": 0,
    "has_unflushed_tracker_data": false
  },
  "breakdown": {
    "text": {
      "request_count": 4,
      "commands": ["chat", "search"],
      "input_tokens": 12450,
      "output_tokens": 6912,
      "cache_read_tokens": 0,
      "cache_write_tokens": 0,
      "reasoning_tokens": 0,
      "estimated_cost_usd": 0.184215
    },
    "image": {
      "request_count": 1,
      "commands": ["image"],
      "input_tokens": 0,
      "output_tokens": 0,
      "cache_read_tokens": 0,
      "cache_write_tokens": 0,
      "reasoning_tokens": 0,
      "estimated_cost_usd": 0.0
    },
    "video": {
      "request_count": 1,
      "commands": ["video"],
      "input_tokens": 0,
      "output_tokens": 0,
      "cache_read_tokens": 0,
      "cache_write_tokens": 0,
      "reasoning_tokens": 0,
      "estimated_cost_usd": 0.0
    },
    "audio": {
      "request_count": 2,
      "commands": ["tts", "stt"],
      "input_tokens": 1200,
      "output_tokens": 300,
      "cache_read_tokens": 0,
      "cache_write_tokens": 0,
      "reasoning_tokens": 0,
      "estimated_cost_usd": 0.0
    }
  },
  "recent_rate_limits": {
    "available": true,
    "captured_at": "2026-05-20T10:08:34Z",
    "provider": "xai-oauth",
    "requests_per_minute": {
      "limit": 60,
      "remaining": 41,
      "used": 19,
      "reset_seconds": 32
    },
    "requests_per_hour": {
      "limit": 1800,
      "remaining": 1712,
      "used": 88,
      "reset_seconds": 1800
    },
    "tokens_per_minute": {
      "limit": 120000,
      "remaining": 108200,
      "used": 11800,
      "reset_seconds": 32
    },
    "tokens_per_hour": {
      "limit": 3000000,
      "remaining": 2971040,
      "used": 28960,
      "reset_seconds": 1800
    }
  }
}
```

### 3.1 结构约束

- `session` 一定存在
- `local_usage` 一定存在
- `breakdown` 一定存在
- `recent_rate_limits` 一定存在，但可 `available: false`

这可以让 SKILL 和自动化围绕本地统计写稳定解析逻辑。

## 4. Account limits 处理结论

当前实现决定：

- 人类可读输出不展示 Account limits
- JSON 输出不返回 `account_limits`
- 当前不对 xAI / xAI OAuth 发起任何 live quota / entitlements 探测

原因：

- 真实实测中，候选端点 `https://api.x.ai/v1/entitlements` 与 `https://api.x.ai/v1/usage` 对当前 OAuth token 返回 `404`
- 对照当前 Hermes 源码，也没有一个已落地的 `xai` / `xai-oauth` account limits 实现可复用
- 继续保留猜测性探测只会制造不稳定行为和误导性输出

因此当前规范改为：

- `usage` 命令只负责本地 session usage
- 若未来拿到 xAI 官方公开且稳定的 quota 接口，再恢复远程探测

## 5. 本地 session / usage 设计

## 5.1 为什么必须补 SQLite session store

当前 `grok-cli` 还没有 Hermes 式的 session 历史层，因此如果直接加一个 `usage` 命令而不补本地存储，它最多只能展示：

- 本次进程中的瞬时计数
- 或单次响应的 `usage`

这做不到用户需要的：

- 当前会话累计输入 / 输出 tokens
- 本地成本估算
- 时长、上下文和压缩历史
- 可重复查询的 session 级历史

所以首版必须把“可查询的 session 账本”一起补上。

## 5.2 推荐存储位置

建议新增：

```text
~/.grok-cli/session.db
```

不建议继续把 usage history 混进 `auth.json`：

- `auth.json` 负责认证状态
- session usage 是追加型历史数据
- SQLite 更适合聚合、回归测试和后续 analytics

## 5.3 推荐依赖

建议新增：

- `rusqlite`

原因：

- 本地 CLI 为主，SQLite 是天然匹配
- 当前代码大多是同步调用，`rusqlite` 比引入完整 async ORM 更合适
- 测试与 fixture 构造简单

金额字段建议内部按 `micro_usd` 整数存储，避免累计浮点误差。

## 5.4 推荐模块结构

```text
src/usage/
  mod.rs
  command.rs
  model.rs
  pricing.rs
  tracker.rs
  sqlite.rs
```

职责建议：

- `command.rs`
  - `usage` 命令组装与输出
- `model.rs`
  - JSON 输出结构、SQLite 记录结构
- `pricing.rs`
  - 模型 pricing table 与成本估算
- `tracker.rs`
  - 进程内累计 usage / rate-limit 快照
- `sqlite.rs`
  - session / event / snapshot 的读写

## 5.5 推荐 SQLite schema

首版至少建议三张表：

### `sessions`

- `session_id TEXT PRIMARY KEY`
- `started_at TEXT NOT NULL`
- `last_activity_at TEXT NOT NULL`
- `provider TEXT NOT NULL`
- `active_model TEXT NULL`
- `request_count INTEGER NOT NULL DEFAULT 0`
- `input_tokens INTEGER NOT NULL DEFAULT 0`
- `output_tokens INTEGER NOT NULL DEFAULT 0`
- `cache_read_tokens INTEGER NOT NULL DEFAULT 0`
- `cache_write_tokens INTEGER NOT NULL DEFAULT 0`
- `reasoning_tokens INTEGER NOT NULL DEFAULT 0`
- `estimated_cost_micro_usd INTEGER NOT NULL DEFAULT 0`
- `context_window_tokens INTEGER NULL`
- `compression_count INTEGER NOT NULL DEFAULT 0`
- `metadata_json TEXT NULL`

### `session_events`

- `event_id TEXT PRIMARY KEY`
- `session_id TEXT NOT NULL`
- `command TEXT NOT NULL`
- `provider TEXT NOT NULL`
- `model TEXT NULL`
- `started_at TEXT NOT NULL`
- `completed_at TEXT NOT NULL`
- `duration_ms INTEGER NOT NULL`
- `input_tokens INTEGER NOT NULL DEFAULT 0`
- `output_tokens INTEGER NOT NULL DEFAULT 0`
- `cache_read_tokens INTEGER NOT NULL DEFAULT 0`
- `cache_write_tokens INTEGER NOT NULL DEFAULT 0`
- `reasoning_tokens INTEGER NOT NULL DEFAULT 0`
- `estimated_cost_micro_usd INTEGER NOT NULL DEFAULT 0`
- `context_window_tokens INTEGER NULL`
- `request_id TEXT NULL`
- `metadata_json TEXT NULL`

### `rate_limit_snapshots`

- `snapshot_id TEXT PRIMARY KEY`
- `session_id TEXT NOT NULL`
- `event_id TEXT NULL`
- `provider TEXT NOT NULL`
- `captured_at TEXT NOT NULL`
- `requests_per_minute_limit INTEGER NULL`
- `requests_per_minute_remaining INTEGER NULL`
- `requests_per_minute_reset_seconds REAL NULL`
- `requests_per_hour_limit INTEGER NULL`
- `requests_per_hour_remaining INTEGER NULL`
- `requests_per_hour_reset_seconds REAL NULL`
- `tokens_per_minute_limit INTEGER NULL`
- `tokens_per_minute_remaining INTEGER NULL`
- `tokens_per_minute_reset_seconds REAL NULL`
- `tokens_per_hour_limit INTEGER NULL`
- `tokens_per_hour_remaining INTEGER NULL`
- `tokens_per_hour_reset_seconds REAL NULL`

说明：

- `sessions` 是聚合视图，给 `usage` 直接读
- `session_events` 是可追溯明细
- `rate_limit_snapshots` 保留最近一次和历史快照，方便后续 `/insights`

## 5.6 active session 解析规则

推荐顺序：

1. 显式 `--session-id`
2. 环境变量 `GROK_CLI_SESSION_ID`
3. SQLite metadata 中的 `active_session_id`
4. 如果都没有，则自动创建新 session

这样有三个好处：

- 直接手跑 CLI 时不需要先手工建 session
- SKILL / 自动化可以显式绑定同一 session
- 后续新增 `session` 命令时不需要推倒重来

## 5.7 哪些命令要写 usage history

首版建议只记录真正会消耗 provider 资源的命令：

- `chat`
- `search`
- `image`
- `video`
- `tts`
- `stt`
- top-level capability commands 命令发往上游的真实请求

不建议首版记录：

- `login`, `status`, `refresh`, `logout`, and hidden auth rescue commands such as `exchange-code`
- `state`
- `usage`

## 6. 本地成本估算

## 6.1 目标

成本估算只做本地计算，不依赖网络。

输入：

- 每次 API 响应里的 `usage`
- 当前请求所用模型
- 本地 bundled pricing table

输出：

- `estimated_cost_usd`
- `pricing_status`
- `pricing_source`

## 6.2 首版建议

- 优先支持 xAI 当前已接入的文本 / 搜索模型
- 图片、视频、TTS、STT 如果暂时缺少稳定公开 pricing，可返回：
  - `estimated_cost_usd: null`
  - `pricing_status: "unknown"`
- 不因为价格表缺失而让命令失败

## 6.3 上下文长度与压缩历史

这两个字段要提前在 schema 里占位：

- `context_window_tokens`
- `compression_count`

当前 `grok-cli` 还没有 Hermes 式自动压缩历史，因此首版允许：

- `context_window_tokens: null`
- `compression_count: 0`

但 session store 和输出 schema 先冻结，避免后面再改接口。

## 7. 推荐实现顺序

1. 冻结 `usage` 命令面和 JSON 输出结构
2. 引入 `src/usage/` 和 SQLite session store
3. 为 top-level capability commands 请求路径补 usage instrumentation
4. 记录并归档最近 rate-limit headers
5. 实现 `usage` 命令的人类输出与 `--json`
6. 增加回归测试与文档样例

## 8. 测试面

首版至少覆盖以下场景：

- 没有 session.db 时，`usage --json` 返回空 session 但不失败
- JSON 和人类可读输出都不包含 Account limits 分组
- rate-limit headers 能被持久化并显示
- SQLite schema 损坏时，返回明确错误码

## 9. 推荐结论

推荐方案是：

- 新增顶层 `usage` 命令
- 新增 `src/usage/` session accounting 层
- 不新增 provider live quota adapter，避免猜测性 xAI quota endpoint 误导用户
- 让 `usage` 固定为“本地统计 + 最近 rate-limit 快照”的可预测命令

这样最贴近 Hermes `/usage` 的目标行为，同时不会把未公开的 xAI quota endpoint 绑死在命令契约里。
