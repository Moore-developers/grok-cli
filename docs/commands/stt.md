# `grok-cli stt`

## 用途

将本地音频文件转写为文本。

## 常用方式

```bash
grok-cli stt ./sample.wav
```

指定语言：

```bash
grok-cli stt ./sample.mp3 --language zh
```

脚本或 SKILL：

```bash
grok-cli stt --json --file ./sample.wav
```

## 参数

- `PATH`：位置参数，要转写的音频文件。
- `--file <PATH>`：脚本友好的显式文件参数。
- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--model <MODEL>`：命令级模型覆盖参数，当前主要用于兼容和 usage 标记。
- `--language <LANG>`：语言代码，默认 `en`。
- `--timeout <SECONDS>`：请求超时，默认 `120` 秒；大文件可显式调大。

## 行为规格

- 本地文件必须存在，否则返回 `invalid_args`。
- 当前 multipart 请求发送 `file`、`format=true`、`language`，与 Hermes 媒体参数形状保持一致。
- 发请求前会检查 access token 是否临近过期，必要时先 refresh。
- 响应会读取 `text` 或 `transcript` 字段。
- 成功后写入本地 usage SQLite 的 audio 分类。

## JSON 输出重点

`data` 中包含：

- `success`
- `provider`
- `credential_source`
- `transcript`

## 相关文档

- [tts](./tts.md)
- [usage](./usage.md)
