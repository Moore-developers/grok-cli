# `grok-cli usage`

## 用途

查看本地 session usage。它会从 SQLite session store 中统计文本、图片、视频、音频等使用记录，并格式化打印 token、成本估算和上下文长度。

默认数据库：

```text
~/.grok-cli/session.db
```

## 常用方式

```bash
grok-cli usage
```

只读本地统计，不尝试 provider account lookup：

```bash
grok-cli usage --local-only
```

说明：`usage` 现在默认就是本地统计，`--local-only` 只作为旧脚本兼容参数保留，不需要日常使用。

脚本或 SKILL：

```bash
grok-cli usage --json
```

指定 session：

```bash
grok-cli usage --session-id sess_01_example
```

## 参数

- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--session-db <PATH>`：覆盖 SQLite session 数据库路径。
- `--session-id <ID>`：读取指定 session，而不是 active session。
- `--timeout <SECONDS>`：隐藏兼容参数；当前不查询账号额度。
- `--local-only`：兼容保留参数；当前只展示本地统计。

## 行为规格

- 本地统计成功时，命令整体成功。
- 文本命令统计 token 和成本估算。
- 图片、视频、音频命令统计请求次数、模型和 rate-limit 快照；没有 token 时显示为 0 或 `n/a`。
- token 显示会按 K / M / B 压缩，例如 `124.8K`、`2.8M`。
- 不查询、不展示、不返回 `Account limits`。

## 人类可读输出

默认输出包含：

- `Session Usage`
- `Usage Breakdown`
- `Session metadata`

## JSON 输出重点

`data` 中包含：

- `provider`
- `session`
- `local_usage`
- `breakdown`
- `recent_rate_limits`

## 相关文档

- [`usage` 深度规格](../reference/usage-command-spec.md)
- [样例输出](../reference/samples.md)
