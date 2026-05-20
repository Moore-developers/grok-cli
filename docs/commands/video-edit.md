# `grok-cli video-edit`

## 用途

使用 Grok Imagine 编辑已有视频。

该命令独立于 [`video`](./video.md) 生成主命令。`video` 负责 text-to-video、image-to-video 和 reference image video；`video-edit` 负责基于已有 MP4 视频做编辑。

## 常用方式

```bash
grok-cli video-edit --video-url https://example.com/source.mp4 --prompt "Give the woman a silver necklace"
```

脚本或 SKILL：

```bash
grok-cli video-edit --json --video-url https://example.com/source.mp4 --prompt "Make the scene more cinematic"
```

## 参数

- `PROMPT`：位置参数，视频编辑提示词。
- `--prompt <PROMPT>`：脚本友好的显式提示词参数。
- `--video-url <URL>`：要编辑的源视频 URL。
- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--model <MODEL>`：仅覆盖本次视频编辑请求的模型。
- `--timeout <SECONDS>`：整体视频轮询等待上限，默认 `600` 秒；单次 HTTP 请求仍限制在媒体请求上限内。

## 行为规格

- 默认模型为 `grok-imagine-video`。
- 请求 `POST /videos/edits`，请求体发送 `video: {"url": ...}`。
- 不发送 `duration`、`aspect_ratio`、`resolution`；编辑输出继承输入视频属性。
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

`modality` 固定为 `edit`。

## 相关文档

- [video](./video.md)
- [image-edit](./image-edit.md)
