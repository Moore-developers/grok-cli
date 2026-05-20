# Grok SKILL 路由方案

这个目录用于整理一套以 OAuth 为主的 Grok SKILL 能力路由设计，目标是让
用户可以通过自己的 OAuth 来使用 Grok 的完整能力，同时在行为层尽量参考
Hermes Agent 当前把 xAI Grok 接入运行时的方式。

这里的重点不是立刻开始写代码，而是先把下面几件事讲清楚：

- Hermes 当前到底能用 Grok 做什么
- 如何把这些能力整理成一个统一的 SKILL 路由层
- 如何允许用户使用自己的 OAuth 来绑定和使用 Grok
- Hermes 在 Grok OAuth 上依赖了哪些参数、值、行为特征和约束
- 将来做成 SKILL 之后，用户说“用 Grok 做某件事”时应该如何触发和续跑

文档列表：

- `SKILL.md`
- `grok-skill-routing-plan.md`
- `capability-matrix.md`
- `oauth-flow.md`
- `constraints-and-risks.md`
- `hermes-grok-oauth-parameter-observations.md`
