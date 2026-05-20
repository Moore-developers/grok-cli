# `grok-cli video`

## 用途

使用 Grok Imagine 生成视频，支持 text-to-video、image-to-video 和 reference image video。

## 常用方式

文本生成视频：

```bash
grok-cli video "Animate a futuristic skyline" --duration 8
```

图片转视频：

```bash
grok-cli video "Make the scene slowly move" --image-url "https://example.com/source.png"
```

参考图视频：

```bash
grok-cli video "Create a product reveal" --reference-image-url "https://example.com/ref-1.png"
```

脚本或 SKILL：

```bash
grok-cli video --json --prompt "Animate a futuristic skyline" --duration 8
```

## 参数

- `PROMPT`：位置参数，视频提示词。
- `--prompt <PROMPT>`：脚本友好的显式提示词参数。
- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--image-url <URL>`：image-to-video 的源图片 URL。
- `--reference-image-url <URL>`：参考图 URL，可重复，最多 7 个。
- `--duration <SECONDS>`：视频时长，范围会被归一化。
- `--aspect-ratio <RATIO>`：比例，支持 `1:1`、`16:9`、`9:16`、`4:3`、`3:4`、`3:2`、`2:3`。
- `--resolution <VALUE>`：分辨率，支持 `480p`、`720p`。
- `--model <MODEL>`：仅覆盖本次视频请求的模型。
- `--timeout <SECONDS>`：整体轮询等待超时，默认 `600` 秒；单次 create / poll HTTP 请求仍固定按 `120` 秒上限处理。

## 行为规格

- 默认模型为 `grok-imagine-video`。
- `grok-cli model` 不管理视频默认模型；如需切换，请直接传 `--model`。
- `--image-url` 不能和 `--reference-image-url` 同时使用。
- `--reference-image-url` 最多 7 个。
- 默认时长为 8 秒，普通视频最大 15 秒，reference image video 最大 10 秒。
- 默认比例为 `16:9`，默认分辨率为 `720p`。
- 先请求 `POST /videos/generations`，再轮询 `GET /videos/{request_id}`。
- 发请求和轮询前都会检查 access token 是否临近过期，必要时先 refresh。
- 成功后写入本地 usage SQLite 的 video 分类。

## JSON 输出重点

`data` 中包含：

- `provider`
- `credential_source`
- `model`
- `video`
- `modality`
- `aspect_ratio`
- `duration`
- `extra.request_id`

## 相关文档

- [image](./image.md)
- [usage](./usage.md)
