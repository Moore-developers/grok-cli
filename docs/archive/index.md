# 归档文档

这里保存早期 Grok SKILL / Rust CLI 设计阶段的历史文档。

这些文档用于追溯设计背景，不再代表当前公开 CLI 的最新用户入口。当前使用方式请优先看：

- [README](../../README.md)
- [中文 README](../../README.zh-CN.md)
- [快速开始](../guides/quickstart.md)
- [命令参考](../commands/index.md)

## 重要提示

- 归档文档里可能仍然出现旧的 `grok-cli auth ...`、`grok-cli task ...`、`proxy`、`debug` 等开发期入口。
- 当前公开 CLI 已经扁平化为一级命令，例如 `grok-cli login`、`grok-cli chat "..."`、`grok-cli search "..."`、`grok-cli usage`。
- 如果归档文档和当前 README / `docs/commands/index.md` 有冲突，以当前 README 和命令参考为准。

## 归档清单

- [早期 Grok SKILL 路由 README](./README.md)
- [早期 SKILL 定义](./SKILL.md)
- [Grok 能力矩阵](./capability-matrix.md)
- [约束与风险](./constraints-and-risks.md)
- [Grok SKILL 能力路由方案](./grok-skill-routing-plan.md)
- [Hermes Grok OAuth 参数观察清单](./hermes-grok-oauth-parameter-observations.md)
- [Grok OAuth 流程](./oauth-flow.md)
- [Rust CLI 命令规格](./rust-cli-command-spec.md)
- [Rust CLI 设计方案](./rust-cli-design.md)
- [Rust CLI 状态文件 Schema](./rust-cli-state-schema.md)
- [技术架构文档](./technical-architecture.md)
