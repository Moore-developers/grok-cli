# `grok-cli tts`

## 用途

将文本转换为语音，并把音频保存为本地文件。

## 常用方式

```bash
grok-cli tts "Hello from Grok"
```

指定声音、语言和输出文件：

```bash
grok-cli tts "你好，我是 Grok" --voice-id eve --language zh --output ./out/grok.mp3
```

脚本或 SKILL：

```bash
grok-cli tts --json --text "Hello from Grok"
```

列出可用声音：

```bash
grok-cli tts --list-voices --json
```

显式控制输出格式：

```bash
grok-cli tts "Hello" --output-format mp3 --sample-rate 24000 --bit-rate 128000
```

## 参数

- `TEXT`：位置参数，要合成的文本。
- `--text <TEXT>`：脚本友好的显式文本参数。
- `--list-voices`：列出可用 TTS voices，不执行语音合成。
- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--voice-id <VOICE>`：声音 id，默认 `eve`。
- `--language <LANG>`：语言代码，默认 `en`；可传 `auto`。
- `--output <PATH>`：输出音频路径。
- `--output-format <FORMAT>`：输出格式，例如 `mp3` 或 `wav`。
- `--sample-rate <HZ>`：输出采样率。
- `--bit-rate <BPS>`：输出码率。
- `--optimize-streaming-latency <MODE>`：透传 xAI TTS 流式延迟优化模式。
- `--text-normalization <MODE>`：透传 xAI TTS 文本归一化模式。
- `--model <MODEL>`：命令级模型覆盖参数，当前主要用于兼容和 usage 标记。
- `--timeout <SECONDS>`：请求超时，默认 `120` 秒。

## 行为规格

- 默认输出路径在 `~/.hermes/cache/audio/audio_cache/` 下。
- 如果输出文件扩展名为 `.wav`，请求中会带 `output_format` 为 wav。
- 如果显式传入 `--output-format`，它必须和 `--output` 文件扩展名一致。
- xAI TTS 请求体发送 `text`、`voice_id`、`language`，并按参数补充 `output_format`、`optimize_streaming_latency`、`text_normalization`。
- `--list-voices` 调用 `GET /v1/tts/voices`，返回 voices 列表，不要求提供文本。
- 发请求前会检查 access token 是否临近过期，必要时先 refresh。
- 成功后写入本地 usage SQLite 的 audio 分类。

## JSON 输出重点

`data` 中包含：

- `success`
- `provider`
- `credential_source`
- `file_path`
- `media_tag`
- `voice_compatible`
- `output_format`

使用 `--list-voices --json` 时，`data` 中包含：

- `success`
- `provider`
- `credential_source`
- `voices`

## 相关文档

- [stt](./stt.md)
- [usage](./usage.md)
