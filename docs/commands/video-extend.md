# `grok-cli video-extend`

## 用途

使用 Grok Imagine 扩展已有视频。

该命令独立于 [`video`](./video.md) 和 [`video-edit`](./video-edit.md)。`video` 负责生成新视频，`video-edit` 负责编辑已有视频，`video-extend` 负责在已有 MP4 视频末尾追加新片段。

## 常用方式

```bash
grok-cli video-extend --video-url https://example.com/source.mp4 --prompt "The camera pans left" --duration 6
```

脚本或 SKILL：

```bash
grok-cli video-extend --json --video-url https://example.com/source.mp4 --prompt "Continue the camera move"
```

## 参数

- `PROMPT`：位置参数，视频扩展提示词。
- `--prompt <PROMPT>`：脚本友好的显式提示词参数。
- `--video-url <URL>`：要扩展的源视频 URL。
- `--duration <SECONDS>`：扩展片段时长，默认 `6` 秒，会被限制到 `2..=10`。
- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--model <MODEL>`：仅覆盖本次视频扩展请求的模型。
- `--timeout <SECONDS>`：整体视频轮询等待上限，默认 `600` 秒；单次 HTTP 请求仍限制在媒体请求上限内。

## 行为规格

- 默认模型为 `grok-imagine-video`。
- 请求 `POST /videos/extensions`，请求体发送 `video: {"url": ...}` 和归一化后的 `duration`。
- 不发送 `aspect_ratio`、`resolution`；扩展输出继承输入视频属性。
- 先读取创建响应中的 `request_id`，再轮询 `GET /videos/{request_id}` 到终态。
- 成功后写入本地 usage SQLite 的 video 分类。

## JSON 输出重点

`data` 中包含：

- `provider`
- `credential_source`
- `model`
- `video`
- `modality`
- `duration`
- `extra.request_id`

`modality` 固定为 `extension`。

## 相关文档

- [video](./video.md)
- [video-edit](./video-edit.md)
