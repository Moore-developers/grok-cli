# `grok-cli chat`

## 用途

通过 Grok Responses API 执行文本聊天、总结、推理和问答。默认会挂载通用 `web_search`，适合查询近期事实。

默认情况下，给人直接使用时会以流式方式逐步打印正文，但不会把底层 SSE 事件直接暴露到屏幕上，也不会打印 `Thinking...` 等状态提示。给脚本、SKILL 或自动化使用时，建议固定加 `--json`，拿稳定的单次结构化结果。

## 常用方式

```bash
grok-cli chat "总结最近 AI 新闻"
```

纯聊天，不使用搜索：

```bash
grok-cli chat "解释一下 OAuth PKCE" --no-web-search
```

同时挂载网页搜索和 X 搜索：

```bash
grok-cli chat "最近 48 小时 AI 圈最热的讨论是什么?" --with-x-search
```

脚本或 SKILL：

```bash
grok-cli chat --json --prompt "Summarize today's AI news"
```

显式关闭默认流式：

```bash
grok-cli chat "讲一个短故事" --no-stream
```

显式声明流式输出：

```bash
grok-cli chat "讲一个短故事" --stream
```

输出原始事件流：

```bash
grok-cli chat "讲一个短故事" --raw-stream
```

## 参数

- `PROMPT`：位置参数，用户提示词。
- `--prompt <PROMPT>`：脚本友好的显式提示词参数。
- `--json`：使用统一 JSON 信封输出。默认会关闭流式，返回稳定单次结果。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--system <TEXT>`：系统指令，映射到 Responses API `instructions`。
- `--model <MODEL>`：仅覆盖本次请求的模型。
- `--no-web-search`：关闭默认 `web_search`。
- `--with-x-search`：额外挂载 `x_search`。
- `--allowed-domain <DOMAIN>`：限制 web search 域名，可重复，最多 10 个。
- `--excluded-domain <DOMAIN>`：排除 web search 域名，可重复，最多 10 个。
- `--enable-image-understanding`：为搜索工具启用图片理解。
- `--allowed-x-handle <HANDLE>`：限制 X 搜索 handle，可重复，最多 10 个。
- `--excluded-x-handle <HANDLE>`：排除 X 搜索 handle，可重复，最多 10 个。
- `--from-date <YYYY-MM-DD>`：X 搜索开始日期。
- `--to-date <YYYY-MM-DD>`：X 搜索结束日期。
- `--enable-video-understanding`：为 X 搜索启用视频理解。
- `--stream`：显式使用格式化流式输出。
- `--no-stream`：关闭默认流式，打印单次最终结果。
- `--raw-stream`：输出原始 normalized stream events，适合调试或程序消费。
- `--timeout <SECONDS>`：请求超时，默认 `3600` 秒。

## 行为规格

- 默认模型为 `grok-4.3`，可被 [`model --model ...`](./model.md) 作为共享文本模型覆盖。
- 默认请求写入 `store: false`。
- 默认附带 `web_search`。
- 非 `--json` 时默认走“人类可读正文流式输出”；不会直接打印底层事件包装。
- `response.created` / reasoning / tool 事件不会打印为人类可见状态；正文只来自 `response.output_text.delta`。
- `--stream` 会显式使用同样的人类可读正文流式输出。
- `--raw-stream` 会切到原始事件流输出。
- `--json` 默认走非流式稳定结果。
- `--with-x-search` 会在默认 `web_search` 之外再挂 `x_search`。
- `--no-web-search --with-x-search` 会只挂 `x_search`。
- 只要挂载工具，就设置 `tool_choice: "auto"` 和 `parallel_tool_calls: true`。
- 非流式响应会写入本地 usage SQLite。
- 原始事件流会输出 `response.output_text.delta`、`response.output_text.done`、`response.output_item.done`、`response.completed`、`response.failed` 等事件。

## JSON 输出重点

非流式 `data` 中包含：

- `provider`
- `model`
- `protocol`
- `output_text`
- `finish_reason`
- `tool_calls`

## 相关文档

- [search](./search.md)
- [model](./model.md)
- [样例输出](../reference/samples.md)
