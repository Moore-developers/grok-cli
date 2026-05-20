# Project Index

这个目录放项目管理和验收资料。它们用于开发推进和质量确认，不属于普通用户主阅读路径。

## 文档列表

1. [验收样例](./acceptance.md)
2. [计划任务清单](../plan-task.md)
3. [SuperGrok 媒体能力补齐计划](./supergrok-media-capability-plan.md)
4. [发布前最终验证计划](./pre-release-validation-plan.md)

## 文档职责

### [验收样例](./acceptance.md)

记录项目当前认为需要验收的主路径：

- 状态与认证
- 浏览器登录
- token refresh
- 文本默认模型切换
- chat
- X search
- 图片 / 视频 / TTS / STT
- usage
- 回归测试

### [计划任务清单](../plan-task.md)

持续维护项目阶段、任务勾选、关键决策和当前状态。它是开发推进用的工作清单，不是公开用户文档。

### [SuperGrok 媒体能力补齐计划](./supergrok-media-capability-plan.md)

记录 `image` / `tts` / `stt` 与 Hermes Agent、xAI 官方文档之间的能力差异，并拆成可执行任务。每个能力补齐项都必须同步补模块级测试、命令级 stub 测试和用户文档。

### [发布前最终验证计划](./pre-release-validation-plan.md)

记录公开推广前的最终确认项：真实媒体测试、SKILL 补全、极简性能分析、安装/OAuth 闭环和安全隐私护栏。

## 使用建议

- 开发新能力前，先看 [计划任务清单](../plan-task.md) 确认当前阶段。
- 扩展 `image` / `tts` / `stt` 前，先看 [SuperGrok 媒体能力补齐计划](./supergrok-media-capability-plan.md) 确认参数范围和测试门槛。
- 改动用户可见行为后，同步检查 [验收样例](./acceptance.md) 是否需要更新。
- 发布前按 [发布与安装指南](../guides/release.md)、[验收样例](./acceptance.md) 和 [发布前最终验证计划](./pre-release-validation-plan.md) 交叉检查。

## 相关入口

- [Guides Index](../guides/index.md)
- [Reference Index](../reference/index.md)
- [CLI 命令索引](../commands/index.md)
- [总索引](../index.md)
