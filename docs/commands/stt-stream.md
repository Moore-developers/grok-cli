# `grok-cli stt-stream`

## 用途

通过 xAI WebSocket STT 接口做实时语音转写。

这是实验入口，不替代批量转写命令 [`stt`](./stt.md)。第一版会把本地音频文件按二进制帧发送到 WebSocket，然后发送 `{"type":"audio.done"}` 结束信号，并持续读取转写事件。

## 常用方式

```bash
grok-cli stt-stream --file ./sample.wav --language en --interim-results
```

输出 JSON 事件汇总：

```bash
grok-cli stt-stream --file ./sample.wav --json
```

使用原始 PCM 参数：

```bash
grok-cli stt-stream --file ./sample.raw --encoding pcm_s16le --sample-rate 16000 --language en
```

## 参数

- `PATH`：位置参数，待转写音频文件。
- `--file <PATH>`：脚本友好的显式音频文件参数。
- `--json`：使用统一 JSON 信封输出，`data.events` 包含收到的事件列表。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--model <MODEL>`：覆盖 streaming STT 模型，默认 `grok-transcribe`。
- `--language <LANG>`：语言代码，默认 `en`。
- `--interim-results`：请求中间转写结果。
- `--endpointing <VALUE>`：透传官方 endpointing 参数。
- `--encoding <ENCODING>`：原始音频编码，例如 `pcm_s16le`。
- `--sample-rate <HZ>`：原始音频采样率。
- `--diarize`：启用说话人分离。
- `--filler-words`：保留 filler words。
- `--multichannel`：按多声道处理。
- `--channels <LIST>`：指定声道，例如 `0,1`。
- `--keyterm <TERM>`：关键词增强，可重复。
- `--timeout <SECONDS>`：预留的 WebSocket 会话超时参数；当前实验入口主要用于保持 CLI 参数兼容。

## 行为规格

- 真实连接地址由 OAuth 状态中的 `base_url` 转成 WebSocket URL，例如 `https://api.x.ai/v1` 会变成 `wss://api.x.ai/v1/stt`。
- Streaming STT 的配置参数放在 URL query 中，`--count` 等图片参数不参与这里。
- 发请求前会检查 access token 是否临近过期，必要时先 refresh。
- 非 JSON 输出按事件打印 `interim: ...` 或 `final: ...`。
- JSON 输出在连接结束后返回事件数组；每个事件保留标准化字段和原始事件。
- 这是 streaming STT，不替代 batch [`stt`](./stt.md) 的文件/URL multipart 转写。

## JSON 输出重点

`data` 中包含：

- `success`
- `provider`
- `credential_source`
- `events`

每个 `events[]` 中包含：

- `event_type`
- `transcript`
- `is_final`
- `raw`

## 相关文档

- [stt](./stt.md)
- [tts](./tts.md)
