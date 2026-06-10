# grok-cli 文档索引

`grok-cli` 是一个 OAuth-first 的 Grok / xAI 本地 CLI，负责浏览器登录、状态管理、Grok 能力调用和本地 usage 统计。

## 推荐阅读路径

1. 新用户先看 [Guides Index](./guides/index.md) 或直接看 [快速开始](./guides/quickstart.md)。
2. 日常查命令看 [CLI 命令索引](./commands/index.md)。
3. 自动化 / SKILL 接入看 [Reference Index](./reference/index.md)、[SKILL 集成约定](./reference/skill-integration.md)，或直接看仓库内置 [`grok-cli` skill](../../skills/grok-cli/SKILL.md) 与 [Skill 安装说明](../../skills/README.md)。安装 skill 时使用 `npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes`。
4. 发布到 GitHub 或给用户安装看 [发布与安装指南](./guides/release.md)，当前采用 SKILL-first；macOS Apple Silicon 可上传本地构建 tarball，macOS Intel 和 Linux 走 source-first，Windows 走 GitHub Release binary。
5. 开发推进和验收看 [Project Index](./project/index.md)，媒体能力补齐看 [SuperGrok 媒体能力补齐计划](./project/supergrok-media-capability-plan.md)。
6. 需要追溯历史设计时再看 [归档文档](./archive/index.md)。

## CLI 命令 Spec

### 认证

- [`login`](./commands/login.md)：打开真实浏览器完成 xAI OAuth 登录。
- [`status`](./commands/status.md)：读取本地 OAuth 状态并判断是否可用。
- [`refresh`](./commands/refresh.md)：使用 refresh token 刷新 access token。
- [`logout`](./commands/logout.md)：删除本地 OAuth 状态。

### 本地状态与模型

- [`state`](./commands/state.md)：查看本地 OAuth 状态的脱敏摘要。
- [`model`](./commands/model.md)：管理 `chat` / `search` 共享默认文本模型。

### 文本能力

- [`chat`](./commands/chat.md)：通过 Grok Responses API 执行聊天，默认带 `web_search`。
- [`search`](./commands/search.md)：通过 Grok `x_search` 搜索 X。

### 媒体能力

- [`image`](./commands/image.md)：使用 Grok Imagine 生成图片。
- [`image-edit`](./commands/image-edit.md)：使用 Grok Imagine 编辑一张或多张参考图片。
- [`video`](./commands/video.md)：使用 Grok Imagine 生成视频。
- [`video-edit`](./commands/video-edit.md)：使用 Grok Imagine 编辑已有视频。
- [`video-extend`](./commands/video-extend.md)：使用 Grok Imagine 扩展已有视频。
- [`tts`](./commands/tts.md)：文本转语音。
- [`stt`](./commands/stt.md)：语音转文字。
- [`stt-stream`](./commands/stt-stream.md)：通过 WebSocket 实验性实时语音转文字。

### 使用统计

- [`usage`](./commands/usage.md)：查看本地 session usage、分类统计和最近 rate-limit 快照。
- [`update`](./commands/update.md)：检查最新 release、升级 CLI，并管理被动更新提示。

## 文档结构

```text
docs/
├─ index.md
├─ commands/
│  ├─ index.md
│  ├─ login.md
│  ├─ status.md
│  ├─ refresh.md
│  ├─ logout.md
│  ├─ state.md
│  ├─ model.md
│  ├─ usage.md
│  ├─ update.md
│  ├─ chat.md
│  ├─ search.md
│  ├─ image.md
│  ├─ image-edit.md
│  ├─ video.md
│  ├─ video-edit.md
│  ├─ video-extend.md
│  ├─ tts.md
│  ├─ stt.md
│  └─ stt-stream.md
├─ guides/
│  ├─ index.md
│  ├─ quickstart.md
│  ├─ troubleshooting.md
│  └─ release.md
├─ reference/
│  ├─ index.md
│  ├─ samples.md
│  ├─ internal-auth.md
│  ├─ usage-command-spec.md
│  ├─ skill-integration.md
│  └─ extension-points.md
├─ project/
│  ├─ index.md
│  ├─ acceptance.md
│  └─ supergrok-media-capability-plan.md
├─ plan-task.md
└─ archive/
   └─ index.md
skills/
└─ grok-cli/
   └─ SKILL.md
```

## 可以合并和简化的方向

- `commands/index.md` 现在保留为简短总览；每个命令的详细参数和输出规格迁移到 `commands/<command>.md`。
- `guides/quickstart.md` 只保留新用户最快路径，不承载完整参数说明。
- `reference/samples.md` 保留 JSON / human 输出样例，不重复完整命令参数。
- `reference/usage-command-spec.md` 是 `usage` 的深度设计文档；日常使用看 [`commands/usage.md`](./commands/usage.md) 即可。
- `project/acceptance.md`、`project/supergrok-media-capability-plan.md` 和 `plan-task.md` 属于项目管理资料，不放在用户主路径里。
- `archive/` 只用于历史追溯，里面出现的旧命令形态不代表当前公开 CLI。

## 分区入口

- [Commands Index](./commands/index.md)：每个公开 CLI 命令的 spec 和用法。
- [Guides Index](./guides/index.md)：快速开始、排障、发布安装。
- [Reference Index](./reference/index.md)：样例输出、稳定契约、内部救援、深度规格。
- [Project Index](./project/index.md)：验收样例、计划任务清单和 SuperGrok 媒体能力补齐计划。
- [Archive Index](./archive/index.md)：历史设计文档。

## 当前公开命令

```text
grok-cli <login|status|refresh|logout|state|model|usage|update|chat|search|image|image-edit|video|video-edit|video-extend|tts|stt|stt-stream> ...
```

## 输出约定

- 给人使用时，推荐位置参数，例如 `grok-cli chat "总结最近 AI 新闻"`。
- `chat` / `search` 面向人类交互时默认流式打印可读正文；如果想关闭，可加 `--no-stream`。
- 给脚本或 SKILL 使用时，推荐 `--json` 和显式参数，例如 `grok-cli chat --json --prompt "..."`。
- 成功与失败都遵循统一 JSON 信封，详见 [SKILL 集成约定](./reference/skill-integration.md) 和 [样例输出](./reference/samples.md)。
- 授权异常救援入口见 [内部认证救援入口](./reference/internal-auth.md)，不属于公开日常命令面。
