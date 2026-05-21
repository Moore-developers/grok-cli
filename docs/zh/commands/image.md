# `grok-cli image`

## 用途

使用 Grok Imagine 生成图片。

## 常用方式

```bash
grok-cli image "A cinematic skyline at sunrise"
```

指定比例和分辨率：

```bash
grok-cli image "A cinematic skyline" --aspect-ratio 16:9 --resolution 1k
```

保存 base64 图片到本地文件：

```bash
grok-cli image "A logo mark" --output-file ./out/logo.png
```

生成多张图片：

```bash
grok-cli image "A cinematic skyline" --count 4 --response-format url --json
```

保存多张 base64 图片到目录：

```bash
grok-cli image "A logo mark" --count 4 --output-dir ./out/logos
```

脚本或 SKILL：

```bash
grok-cli image --json --prompt "A cinematic skyline"
```

## 参数

- `PROMPT`：位置参数，图片提示词。
- `--prompt <PROMPT>`：脚本友好的显式提示词参数。
- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--model <MODEL>`：仅覆盖本次图片请求的模型。
- `--aspect-ratio <RATIO>`：输出比例，例如 `16:9` 或 `1:1`。
- `--resolution <VALUE>`：输出分辨率，例如 `1k`。
- `--count <N>`：生成图片数量，范围 `1..=10`，默认 `1`；请求体会映射为 xAI 官方字段 `n`。
- `--response-format <url|b64_json>`：显式控制图片返回 URL 或 base64。
- `--output-file <PATH>`：要求上游返回 base64，并保存为本地文件。
- `--output-dir <PATH>`：要求上游返回 base64，并把多张图片保存为 `image-001.png`、`image-002.png` 等文件。
- `--timeout <SECONDS>`：请求超时，默认 `120` 秒。

## 行为规格

- 默认模型为 `grok-imagine-image`。
- `grok-cli model` 不管理图片默认模型；如需切换，请直接传 `--model`。
- 发请求前会检查 access token 是否临近过期，必要时先 refresh。
- 默认请求 `n=1`，不显式发送 `response_format`。
- `--count` 超出 `1..=10` 会返回 `invalid_args`。
- 未传 `--output-file` 时，返回图片 URL 或 data URL。
- 传 `--output-file` 时，保存解码后的图片文件，并在输出中返回本地路径。
- `--output-file` 只支持单图；多图落盘请使用 `--output-dir`。
- `--output-file` 和 `--output-dir` 会隐式使用 `response_format=b64_json`。
- 如果显式传 `--response-format url`，不能同时使用 `--output-file` 或 `--output-dir`。
- 成功后写入本地 usage SQLite 的 image 分类。

## JSON 输出重点

`data` 中包含：

- `provider`
- `credential_source`
- `model`
- `image`
- `images`
- `aspect_ratio`
- `extra`

兼容性说明：
- `image` 始终保留，表示第一张图片或第一张本地落盘路径。
- `images` 返回完整列表；单图时也是只包含一个元素的数组。

## 相关文档

- [video](./video.md)
- [usage](./usage.md)
