# CLI 命令索引

这个目录保存每个公开 CLI 命令的 spec 和使用说明。日常使用优先看这里；更深的输出样例和内部设计放在 [`../reference/`](../reference/)。

## 顶层命令

```text
grok-cli <login|status|refresh|logout|state|model|usage|chat|search|image|video|tts|stt|stt-stream> ...
```

## 认证命令

| 命令 | 文档 | 用途 |
| --- | --- | --- |
| `login` | [`login.md`](./login.md) | 打开真实浏览器完成 xAI OAuth 登录，并保存 token。 |
| `status` | [`status.md`](./status.md) | 读取本地 OAuth 状态，判断是否已登录、是否需要重登。 |
| `refresh` | [`refresh.md`](./refresh.md) | 使用 refresh token 刷新 access token。 |
| `logout` | [`logout.md`](./logout.md) | 删除本地 OAuth 状态。 |

## 状态与模型

| 命令 | 文档 | 用途 |
| --- | --- | --- |
| `state` | [`state.md`](./state.md) | 查看本地 OAuth 状态的脱敏摘要。 |
| `model` | [`model.md`](./model.md) | 管理 `chat` / `search` 共享默认文本模型。 |

## 文本能力

| 命令 | 文档 | 用途 |
| --- | --- | --- |
| `chat` | [`chat.md`](./chat.md) | 执行 Grok 文本聊天，默认带通用 `web_search`。 |
| `search` | [`search.md`](./search.md) | 使用 Grok `x_search` 搜索 X。 |

## 媒体能力

| 命令 | 文档 | 用途 |
| --- | --- | --- |
| `image` | [`image.md`](./image.md) | 使用 Grok Imagine 生成图片。 |
| `video` | [`video.md`](./video.md) | 使用 Grok Imagine 生成视频。 |
| `tts` | [`tts.md`](./tts.md) | 文本转语音，并保存本地音频文件。 |
| `stt` | [`stt.md`](./stt.md) | 将本地音频文件转写为文本。 |
| `stt-stream` | [`stt-stream.md`](./stt-stream.md) | 通过 WebSocket 实验性实时转写本地音频。 |

## 使用统计

| 命令 | 文档 | 用途 |
| --- | --- | --- |
| `usage` | [`usage.md`](./usage.md) | 查看本地 session usage、分类统计、成本估算和最近 rate-limit 快照。 |

## 通用约定

- 人类使用优先位置参数，例如 `grok-cli chat "总结最近 AI 新闻"`。
- `chat` / `search` 给人直接使用时默认流式打印可读正文；如果要单次结果可加 `--no-stream`，如果要原始事件流可加 `--raw-stream`。
- 脚本、SKILL、自动化使用优先 `--json` 和显式参数，例如 `grok-cli chat --json --prompt "..."`。
- 所有命令的 `--json` 成功输出使用 `{ ok, command, data }` 信封。
- 所有命令的 `--json` 失败输出使用 `{ ok, command, error }` 信封。
- 大多数需要 OAuth 的命令支持 `--auth-file <PATH>`，用于测试、隔离状态或多账号场景。
- 内部授权救援入口不列入公开命令索引，详见 [`../reference/internal-auth.md`](../reference/internal-auth.md)。

## 常用工作流

```bash
grok-cli login
grok-cli status
grok-cli chat "总结最近 AI 新闻"
grok-cli search "What are builders saying about Grok today?"
grok-cli image "A cinematic skyline at sunrise"
grok-cli video "Animate a futuristic skyline" --duration 8
grok-cli tts "Hello from Grok"
grok-cli stt ./sample.wav
grok-cli stt-stream ./sample.wav --interim-results
grok-cli usage
```

脚本模式：

```bash
grok-cli status --json
grok-cli chat --json --prompt "Summarize today's AI news"
grok-cli search --json --query "Grok Hermes latest updates"
grok-cli usage --json
```
