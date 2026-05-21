# `grok-cli image-edit`

## 用途

使用 Grok Imagine 编辑一张或多张参考图片。

该命令独立于 [`image`](./image.md) 生成主命令，避免把“纯生成”和“基于参考图编辑”混在同一个入口里。

## 常用方式

编辑单张图片：

```bash
grok-cli image-edit --image ./source.png --prompt "Make it cinematic"
```

使用远程图片 URL：

```bash
grok-cli image-edit --image https://example.com/source.png --prompt "Change the background to sunset"
```

多图编辑，最多 3 张：

```bash
grok-cli image-edit \
  --image ./a.png \
  --image ./b.png \
  --image ./c.png \
  --prompt "Blend these references into one editorial image"
```

保存 base64 编辑结果：

```bash
grok-cli image-edit --image ./source.png --prompt "Make it cinematic" --output-file ./out/edited.png
```

## 参数

- `PROMPT`：位置参数，编辑提示词。
- `--prompt <PROMPT>`：脚本友好的显式提示词参数。
- `--image <PATH_OR_URL>`：输入图片，可重复，最多 3 张；支持本地路径、`http(s)` URL 或 data URI。
- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--model <MODEL>`：仅覆盖本次图片编辑请求的模型。
- `--aspect-ratio <RATIO>`：输出比例，例如 `16:9` 或 `1:1`。
- `--resolution <VALUE>`：输出分辨率，例如 `1k`。
- `--response-format <url|b64_json>`：显式控制图片返回 URL 或 base64。
- `--output-file <PATH>`：要求上游返回 base64，并保存为本地文件。
- `--timeout <SECONDS>`：请求超时，默认 `120` 秒。

## 行为规格

- 默认模型为 `grok-imagine-image`。
- 单张图片请求发送官方字段 `image`，多张图片请求发送官方字段 `images`。
- 本地图片会编码成 `data:image/<ext>;base64,...` 后作为 image URL 输入。
- `--image` 超过 3 个会返回 `invalid_args`。
- `--output-file` 会隐式使用 `response_format=b64_json`。
- 如果显式传 `--response-format url`，不能同时使用 `--output-file`。
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
- `image` 表示第一张编辑结果或本地落盘路径。
- `images` 返回完整编辑结果列表；单图时也是只包含一个元素的数组。

## 相关文档

- [image](./image.md)
- [video](./video.md)
