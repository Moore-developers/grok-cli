# `grok-cli stt`

## 用途

将本地音频文件或远程音频 URL 转写为文本。

## 常用方式

```bash
grok-cli stt ./sample.wav
```

指定语言：

```bash
grok-cli stt ./sample.mp3 --language zh
```

转写远程音频：

```bash
grok-cli stt --url https://example.com/sample.wav --language auto
```

高级转写参数：

```bash
grok-cli stt ./meeting.wav --diarize --keyterm Grok --keyterm xAI --filler-words
```

脚本或 SKILL：

```bash
grok-cli stt --json --file ./sample.wav
```

## 参数

- `PATH`：位置参数，要转写的音频文件。
- `--file <PATH>`：脚本友好的显式文件参数。
- `--url <URL>`：转写远程音频 URL；不能和 `PATH` / `--file` 同时使用。
- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--model <MODEL>`：命令级模型覆盖参数，当前主要用于兼容和 usage 标记。
- `--language <LANG>`：语言代码，默认 `en`。
- `--format <true|false>`：是否请求格式化转写文本，默认 `true`。
- `--audio-format <FORMAT>`：当音频没有可识别容器元数据时，显式指定原始音频格式。
- `--sample-rate <HZ>`：原始音频采样率。
- `--multichannel`：按多声道音频处理。
- `--channels <CHANNELS>`：指定要转写的声道，例如 `0,1`。
- `--diarize`：启用说话人分离。
- `--keyterm <TERM>`：关键词提示，可重复传入。
- `--filler-words`：保留填充词。
- `--timeout <SECONDS>`：请求超时，默认 `120` 秒；大文件可显式调大。

## 行为规格

- 必须提供 `PATH`、`--file` 或 `--url` 之一。
- `--url` 不能和本地文件输入同时使用。
- 本地文件必须存在，否则返回 `invalid_args`。
- multipart 请求会发送 `file` 或 `url`，并按参数补充 `format`、`language`、`audio_format`、`sample_rate`、`multichannel`、`channels`、`diarize`、`keyterm`、`filler_words`。
- 发请求前会检查 access token 是否临近过期，必要时先 refresh。
- 响应会读取 `text` 或 `transcript` 字段。
- 如果上游返回 `language`、`duration`、`words`、`channels`，`--json` 会保留这些结构化字段。
- 成功后写入本地 usage SQLite 的 audio 分类。

## JSON 输出重点

`data` 中包含：

- `success`
- `provider`
- `credential_source`
- `transcript`
- `language`
- `duration`
- `words`
- `channels`

## 相关文档

- [tts](./tts.md)
- [usage](./usage.md)
