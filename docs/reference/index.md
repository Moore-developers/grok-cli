# Reference Index

这个目录放稳定契约、样例输出和内部设计说明。它们偏“是什么”和“为什么这样设计”，不是新用户最短上手路径。

## 文档列表

1. [示例状态与样例输出](./samples.md)
2. [内部认证救援入口](./internal-auth.md)
3. [`usage` 命令规格](./usage-command-spec.md)
4. [SKILL 集成约定](./skill-integration.md)
5. [后续能力扩展接口](./extension-points.md)

## 文档职责

### [示例状态与样例输出](./samples.md)

集中保存 JSON 和 human-readable 输出样例，方便上层脚本、SKILL 或文档读者对照字段。

### [内部认证救援入口](./internal-auth.md)

记录不出现在公开 help / README 中的认证救援能力：

- `print-authorize-url` 已移除公开入口
- `exchange-code` 隐藏保留，用于异常授权救援
- 普通用户应优先使用 `grok-cli login`

### [`usage` 命令规格](./usage-command-spec.md)

保存 `usage` 的深度设计：

- JSON 输出结构
- SQLite session store
- session events
- rate-limit snapshots
- 本地成本估算

日常使用请看 [commands/usage.md](../commands/usage.md)。

### [SKILL 集成约定](./skill-integration.md)

给上层自动化和 SKILL 使用，重点是：

- 什么时候用 `--json`
- 统一成功 / 失败信封
- 错误码处理建议
- 认证后如何恢复原始任务

### [后续能力扩展接口](./extension-points.md)

给后续开发使用，说明新增能力时应复用哪些稳定边界：

- 命令分层
- 统一输出契约
- runtime credentials resolver
- upstream 执行层
- 回归测试要求

## 相关入口

- [CLI 命令索引](../commands/index.md)
- [Guides Index](../guides/index.md)
- [Project Index](../project/index.md)
- [总索引](../index.md)
