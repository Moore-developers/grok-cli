# `grok-cli search`

## 用途

通过 Grok Responses API 的 `x_search` 工具搜索 X。它是 X 平台专用入口，和 `chat` 默认的通用 `web_search` 分层存在。

默认情况下，给人直接使用时会以流式方式逐步打印正文，但不会把底层 SSE 事件直接暴露到屏幕上，也不会打印 `Searching X...` 等状态提示。给脚本、SKILL 或自动化使用时，建议固定加 `--json`，拿稳定的单次结构化结果。

## 常用方式

```bash
grok-cli search "What are builders saying about Grok today?"
```

限制时间范围：

```bash
grok-cli search "AI news" --from-date 2026-05-18 --to-date 2026-05-20
```

限制或排除 handle：

```bash
grok-cli search "Grok" --allowed-x-handle xai --excluded-x-handle example
```

脚本或 SKILL：

```bash
grok-cli search --json --query "Grok Hermes latest updates"
```

显式关闭默认流式：

```bash
grok-cli search "AI news" --no-stream
```

输出原始事件流：

```bash
grok-cli search "AI news" --raw-stream
```

## 参数

- `QUERY`：位置参数，搜索 query。
- `--query <QUERY>`：脚本友好的显式 query 参数。
- `--json`：使用统一 JSON 信封输出。默认会关闭流式，返回稳定单次结果。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--allowed-x-handle <HANDLE>`：限制 X handle，可重复，最多 10 个。
- `--excluded-x-handle <HANDLE>`：排除 X handle，可重复，最多 10 个。
- `--from-date <YYYY-MM-DD>`：开始日期。
- `--to-date <YYYY-MM-DD>`：结束日期。
- `--enable-image-understanding`：启用图片理解。
- `--enable-video-understanding`：启用视频理解。
- `--model <MODEL>`：仅覆盖本次请求的模型。
- `--stream`：显式使用格式化流式输出。
- `--no-stream`：关闭默认流式，打印单次最终结果。
- `--raw-stream`：输出原始 normalized stream events，适合调试或程序消费。
- `--timeout <SECONDS>`：请求超时，默认 `3600` 秒。

## 行为规格

- 默认模型为 `grok-4.20-reasoning`，可被 [`model --model ...`](./model.md) 作为共享文本模型覆盖。
- 请求工具固定为 `x_search`。
- 非 `--json` 时默认走“人类可读正文流式输出”；不会直接打印底层事件包装。
- `response.created` / `x_search` tool 事件不会打印为人类可见状态；正文只来自 `response.output_text.delta`。
- `--stream` 会显式使用同样的人类可读正文流式输出。
- `--raw-stream` 会切到原始事件流输出。
- `--json` 默认走非流式稳定结果。
- 请求写入 `tool_choice: "auto"`、`parallel_tool_calls: true`、`store: false`。
- 响应会提取 message answer、citations 和 inline citations。
- 成功后写入本地 usage SQLite。

## JSON 输出重点

`data` 中包含：

- `success`
- `provider`
- `credential_source`
- `tool`
- `model`
- `query`
- `answer`
- `citations`
- `inline_citations`

## 相关文档

- [chat](./chat.md)
- [model](./model.md)
- [样例输出](../reference/samples.md)
