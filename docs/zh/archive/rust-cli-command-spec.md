# Rust CLI 命令规格

## 目标

定义 `grok-cli` 的命令接口、参数、输入输出格式和错误约定。

这份文档以 `Hermes` 当前对 Grok / xAI OAuth 的实现行为为主参考，不再保留“以后再定”的协议空位。后续 Rust 实现应优先复用这些行为边界，而不是重新设计一套新的 Grok CLI 语义。

## 总体原则

### 1. 命令名

统一使用：

```text
grok-cli
```

### 2. 输出协议

所有核心命令必须支持 `--json`。

在 `--json` 模式下：

- 标准输出只输出结构化 JSON，或者在显式流式模式下输出结构化事件流
- 标准错误用于调试日志和人类可读信息

### 3. 退出码约定

建议统一：

- `0`：成功
- `1`：通用执行失败
- `2`：参数错误
- `3`：认证缺失或需要重新登录
- `4`：权限 / entitlement 被拒绝
- `5`：能力面与模型不匹配

实现时仍应以 JSON 错误码为主，退出码为辅助。

### 4. 统一 JSON 信封

成功：

```json
{
  "ok": true,
  "command": "auth status",
  "data": {}
}
```

失败：

```json
{
  "ok": false,
  "command": "auth refresh",
  "error": {
    "code": "auth_relogin_required",
    "message": "需要重新登录",
    "relogin_required": true,
    "entitlement_denied": false
  }
}
```

## 顶层命令结构

```text
grok-cli <subcommand> [options]
```

一级子命令：

- `auth`
- `task`
- `proxy`
- `state`
- `usage`
- `debug`

说明：

- 这里刻意将 `proxy` 作为一级子命令，而不是 `task proxy`
- 原因是 `Hermes` 里的 proxy 本身就是独立运维面，和自然语言任务执行面分离

## Hermes 参考结论

以下结论已经按 `Hermes` 当前代码和文档收口：

- Grok 主聊天运行时：`codex_responses`
- runtime 自动识别：`api.x.ai` => `codex_responses`
- xAI proxy 允许路径：
  - `/responses`
  - `/chat/completions`
  - `/completions`
  - `/embeddings`
  - `/models`
- `Hermes proxy` 当前已有命令面：
  - `proxy start`
  - `proxy status`
  - `proxy providers`
- `Hermes` 的 `/v1/responses` 与 `/v1/chat/completions` 都支持流式
- `Hermes` 的 `x_search` 底层走 `POST https://api.x.ai/v1/responses`
- `x_search` 返回字段已经稳定体现为：
  - `answer`
  - `citations`
  - `inline_citations`
  - `credential_source`
- 图片生成：
  - Hermes 可能返回本地缓存图片路径
  - 也可能直接返回远程 URL
- 视频生成：
  - Hermes 当前直接返回 xAI CDN `video url`
  - 不是先下载成固定本地 `mp4`
- TTS：
  - Hermes 当前实际默认输出目录是 `~/.hermes/cache/audio/audio_cache`
  - 返回结构里主字段是 `file_path`
- STT：
  - Hermes 返回 `{ success, transcript, provider, error? }`

## Grok 格式矩阵

这里区分三层概念：

### 1. 传输方式

- `stream`
- `non-stream`

说明：

- `stream` 不是独立协议格式，而是响应传输方式
- 对 Grok 而言，聊天类和代理兼容层都可能支持流式

### 2. 协议格式

Hermes 当前和 Grok 相关的主要协议格式：

- `codex_responses`
- `chat_completions`
- `completions`
- `embeddings`
- `models`

说明：

- `codex_responses` 是 Hermes 对 xAI Grok 的主运行时协议
- `chat_completions`、`completions`、`embeddings`、`models` 主要出现在 OpenAI-compatible proxy 暴露面

### 3. 能力格式

从能力面角度，Grok 还包括：

- `x_search`
- `image_gen`
- `video_gen`
- `tts`
- `stt`

这些不是通用聊天协议，而是建立在 xAI 特定接口之上的能力表面。

## 当前支持矩阵

| 能力 | 协议格式 | 传输方式 | 参考 Hermes 行为 |
| --- | --- | --- | --- |
| `task chat` | `codex_responses` | `stream` / `non-stream` | Grok 主路径 |
| `proxy /responses` | `codex_responses` | `stream` / `non-stream` | xAI adapter 允许 |
| `proxy /chat/completions` | `chat_completions` | `stream` / `non-stream` | xAI adapter 允许 |
| `proxy /completions` | `completions` | `non-stream` 为主 | xAI adapter 允许 |
| `proxy /embeddings` | `embeddings` | `non-stream` | xAI adapter 允许 |
| `proxy /models` | `models` | `non-stream` | xAI adapter 允许 |
| `task x-search` | `codex_responses` | `non-stream` | 调 `responses` + `x_search` tool |
| `task image-gen` | xAI 图像接口 | `non-stream` | `/images/generations` |
| `task video-gen` | xAI 视频接口 | 轮询式 | `/videos/generations` + `/videos/{id}` |
| `task tts` | xAI 音频接口 | `non-stream` | 本地音频文件输出 |
| `task stt` | xAI 音频接口 | `non-stream` | 直接返回 transcript |

## 命令要求

命令设计必须明确：

- 哪些命令支持 `--stream`
- 哪些命令永远只返回最终结果
- 哪些命令属于 OpenAI-compatible 暴露面
- 哪些命令属于 xAI 专用能力面

按 Hermes 收口后的规则：

- `task chat`：支持 `--stream`
- `task x-search`：只定义为最终结果型命令
- `task image-gen`：只返回最终结果
- `task video-gen`：只返回最终结果
- `task tts`：只返回最终结果
- `task stt`：只返回最终结果
- `proxy`：独立于 `task`，用于兼容层暴露与本地转发

## `auth` 命令组

### `grok-cli auth status`

#### 用途

检查当前 Grok OAuth 状态，供 SKILL 判断是否需要先登录或刷新。

#### 参数

- `--json`
- `--auth-file <path>` 可选，自定义认证状态文件路径

#### 成功输出

```json
{
  "ok": true,
  "command": "auth status",
  "data": {
    "logged_in": true,
    "provider": "xai-oauth",
    "auth_mode": "oauth_pkce",
    "access_token_present": true,
    "refresh_token_present": true,
    "access_token_expiring": false,
    "relogin_required": false,
    "entitlement_denied": false,
    "last_refresh": "2026-05-19T17:00:00Z",
    "auth_store_path": "/abs/path/auth.json",
    "base_url": "https://api.x.ai/v1"
  }
}
```

#### 失败场景

- 状态文件损坏
- 状态文件不可读
- schema 不合法

#### 典型错误码

- `state_file_missing`
- `state_file_invalid`
- `io_error`

### `grok-cli auth login`

#### 用途

发起完整 Grok OAuth 登录流程。

#### 参数

- `--json`
- `--no-browser`
- `--manual-paste`
- `--timeout <seconds>`
- `--auth-file <path>`
- `--port <port>`

#### 行为说明

- 默认使用浏览器 loopback 模式
- `--no-browser` 适合远程环境
- `--manual-paste` 适合浏览器回贴模式
- 成功后写入状态文件
- 默认只针对 `xai-oauth`
- 不在认证命令里静默回退到 API key

#### 成功输出

```json
{
  "ok": true,
  "command": "auth login",
  "data": {
    "provider": "xai-oauth",
    "auth_mode": "oauth_pkce",
    "saved": true,
    "auth_store_path": "/abs/path/auth.json",
    "redirect_uri": "http://127.0.0.1:56121/callback",
    "base_url": "https://api.x.ai/v1"
  }
}
```

#### 典型错误码

- `auth_callback_timeout`
- `auth_state_mismatch`
- `auth_token_exchange_failed`
- `xai_oauth_tier_denied`
- `io_error`

### `grok-cli auth refresh`

#### 用途

强制刷新当前 access token。

#### 参数

- `--json`
- `--auth-file <path>`

#### 成功输出

```json
{
  "ok": true,
  "command": "auth refresh",
  "data": {
    "provider": "xai-oauth",
    "refreshed": true,
    "last_refresh": "2026-05-19T17:05:00Z"
  }
}
```

#### 典型错误码

- `auth_missing`
- `auth_relogin_required`
- `xai_oauth_tier_denied`
- `auth_refresh_failed`

### `grok-cli auth logout`

#### 用途

清除本地保存的 Grok OAuth 状态。

#### 参数

- `--json`
- `--auth-file <path>`

#### 成功输出

```json
{
  "ok": true,
  "command": "auth logout",
  "data": {
    "removed": true,
    "auth_store_path": "/abs/path/auth.json"
  }
}
```

### `grok-cli auth print-authorize-url`

#### 用途

只生成并输出 authorize URL，不执行完整登录。

适用于：

- 需要由 SKILL 单独控制用户提示
- 需要拆分登录步骤
- 调试 authorize 参数

#### 参数

- `--json`
- `--manual-paste`
- `--auth-file <path>`
- `--port <port>`

#### 成功输出

```json
{
  "ok": true,
  "command": "auth print-authorize-url",
  "data": {
    "authorize_url": "https://auth.x.ai/...",
    "redirect_uri": "http://127.0.0.1:56121/callback",
    "state": "<opaque>",
    "nonce": "<opaque>",
    "pkce_method": "S256"
  }
}
```

说明：

- `state` 与 `nonce` 可以给上层消费
- 不应在普通用户文案中直接暴露

### `grok-cli auth exchange-code`

#### 用途

将已有授权码交换为 token，并写入状态文件。

适用于拆分式 OAuth 流程。

#### 参数

- `--json`
- `--code <authorization_code_or_callback_url>`
- `--state <state>` 可选；未提供时，默认复用本地 pending OAuth 会话中的 `state`
- `--auth-file <path>`

#### 成功输出

与 `auth login` 成功输出类似。

说明：

- `--code` 支持直接传浏览器页复制出的授权码
- `--code` 也支持传完整 callback URL，例如 `http://127.0.0.1:56121/callback?code=...&state=...`
- 如果浏览器页没有给回完整 callback URL，只给单独 code，CLI 会自动复用本地 pending OAuth 会话中的 `state`

## `task` 命令组

### 通用约定

所有 `task` 子命令都建议支持：

- `--json`
- `--auth-file <path>`
- `--model <model>`

所有 `task` 命令都默认要求 OAuth 已存在且可用。

如果认证缺失，应返回结构化错误，而不是静默进入登录。

## `usage` 命令组

### `grok-cli usage`

#### 用途

查看当前会话的本地累计 token / 成本 / 时长统计，并返回预留好的 `account_limits` 结构。

#### 参数

- `--json`
- `--auth-file <path>`
- `--session-db <path>`
- `--session-id <id>`
- `--timeout <seconds>`
- `--local-only`

#### 设计原则

- `usage` 是顶层命令，不挂到 `auth`、`state` 或 `task`
- 本地统计优先
- `account_limits` 当前保留字段位，但不对 xAI 做远程 quota 探测
- 本地统计基于 SQLite session history + 进程内 usage tracker

#### 成功输出

```json
{
  "ok": true,
  "command": "usage",
  "data": {
    "provider": "xai-oauth",
    "session": {
      "session_id": "sess_01_example",
      "started_at": "2026-05-20T10:00:00Z",
      "last_activity_at": "2026-05-20T10:08:34Z",
      "duration_seconds": 514,
      "request_count": 6,
      "session_store_path": "/abs/path/session.db"
    },
    "local_usage": {
      "input_tokens": 12450,
      "output_tokens": 6912,
      "total_tokens": 19362,
      "estimated_cost_usd": 0.184215,
      "pricing_status": "estimated"
    },
    "recent_rate_limits": {
      "available": true
    },
    "account_limits": {
      "available": true,
      "plan": "SuperGrok",
      "windows": []
    }
  }
}
```

#### 降级输出

当账号额度功能未启用时：

```json
{
  "ok": true,
  "command": "usage",
  "data": {
    "provider": "xai-oauth",
    "session": {},
    "local_usage": {},
    "recent_rate_limits": {
      "available": false
    },
    "account_limits": {
      "available": false,
      "message": "Account limits unavailable: xAI live quota probing is disabled"
    }
  }
}
```

#### 首版实现边界

- `xai` / `xai-oauth` 当前不做 provider-specific live quota 探测
- 价格表缺失时，允许 `estimated_cost_usd = null`
- 当前没有 compression runtime 时，`compression_count` 可先固定为 `0`
- 当前没有稳定上下文长度来源时，`context_window_tokens` 可先为 `null`

### `grok-cli task chat`

#### 用途

执行 Grok 聊天 / 推理 / 总结类任务。

#### 参数

- `--json`
- `--prompt <text>`
- `--system <text>` 可选
- `--model <model>`
- `--stream`
- `--timeout <seconds>` 可选
- `--auth-file <path>`

#### 非流式成功输出

```json
{
  "ok": true,
  "command": "task chat",
  "data": {
    "provider": "xai-oauth",
    "model": "grok-4.3",
    "protocol": "codex_responses",
    "output_text": "...",
    "finish_reason": "stop",
    "tool_calls": []
  }
}
```

#### 非流式 tool-calling 成功输出

这里参考 Hermes 对 `Responses` → `chat.completions` / 内部归一化层的包装方式：

- 如果结果中出现 `function_call`
- 则 `finish_reason` 应归一成 `tool_calls`
- `tool_calls` 保留 OpenAI 风格结构

```json
{
  "ok": true,
  "command": "task chat",
  "data": {
    "provider": "xai-oauth",
    "model": "grok-4.3",
    "protocol": "codex_responses",
    "output_text": "",
    "finish_reason": "tool_calls",
    "tool_calls": [
      {
        "id": "call_abc123",
        "type": "function",
        "function": {
          "name": "terminal",
          "arguments": "{\"command\":\"ls\"}"
        }
      }
    ]
  }
}
```

#### 流式约定

当启用 `--stream` 时，不使用 JSON 行协议，直接参考 Hermes 的 SSE 行为。

也就是说：

- 标准输出输出 `text/event-stream`
- 事件体格式采用：

```text
event: <event_type>
data: <json>

```

#### `task chat --stream` 事件集合

最小必需事件：

- `response.output_text.delta`
- `response.output_text.done`
- `response.output_item.done`
- `response.completed`
- `response.failed`

说明：

- 当流中出现工具调用时，`response.output_item.done` 里应能承载 `function_call`
- 当流最终结束时，`response.completed` 里应带完整 `response` 信封
- 如果失败，则发 `response.failed`

#### `response.output_item.done` 的消息项示例

```json
{
  "type": "response.output_item.done",
  "output_index": 0,
  "item": {
    "id": "msg_123",
    "type": "message",
    "status": "completed",
    "role": "assistant",
    "content": [
      {
        "type": "output_text",
        "text": "hello"
      }
    ]
  }
}
```

#### `response.output_item.done` 的工具调用项示例

```json
{
  "type": "response.output_item.done",
  "output_index": 0,
  "item": {
    "type": "function_call",
    "call_id": "call_abc123",
    "name": "terminal",
    "arguments": "{\"command\":\"ls\"}",
    "status": "completed"
  }
}
```

#### `response.completed` 示例

```json
{
  "type": "response.completed",
  "response": {
    "status": "completed",
    "output": [
      {
        "type": "message",
        "role": "assistant",
        "content": [
          {
            "type": "output_text",
            "text": "hello"
          }
        ]
      }
    ]
  }
}
```

#### 典型错误码

- `auth_missing`
- `auth_relogin_required`
- `model_capability_mismatch`
- `request_failed`

### `grok-cli task x-search`

#### 用途

执行 Grok 的 X 搜索能力。

#### 参数

- `--json`
- `--query <text>`
- `--allowed-x-handle <handle>` 可重复，最多 10 个
- `--excluded-x-handle <handle>` 可重复，最多 10 个
- `--from-date <YYYY-MM-DD>`
- `--to-date <YYYY-MM-DD>`
- `--enable-image-understanding`
- `--enable-video-understanding`
- `--model <model>`
- `--timeout <seconds>`
- `--auth-file <path>`

说明：

- 这里使用 `excluded`，而不是 `blocked`
- 因为 Hermes 当前工具字段名就是 `excluded_x_handles`

#### 成功输出

```json
{
  "ok": true,
  "command": "task x-search",
  "data": {
    "success": true,
    "provider": "xai",
    "credential_source": "xai-oauth",
    "tool": "x_search",
    "model": "grok-4.20-reasoning",
    "query": "AI infra discourse",
    "answer": "...",
    "citations": [],
    "inline_citations": []
  }
}
```

#### 传输说明

- `x_search` 底层依赖 `Responses API`
- 但 CLI 层固定定义为最终结果型命令
- 不定义 `--stream`

#### 典型错误码

- `auth_missing`
- `auth_relogin_required`
- `model_capability_mismatch`
- `x_search_not_enabled`
- `request_failed`

### `grok-cli task image-gen`

#### 参数

- `--json`
- `--prompt <text>`
- `--model <model>`
- `--aspect-ratio <ratio>`
- `--resolution <resolution>`
- `--auth-file <path>`

#### 成功输出

参考 Hermes 当前实现，这里不强制只返回本地路径，因为 Hermes 本身支持两种返回：

- 本地缓存文件路径
- 远程 URL

```json
{
  "ok": true,
  "command": "task image-gen",
  "data": {
    "provider": "xai",
    "model": "grok-imagine-image",
    "image": "/abs/path/image.png",
    "aspect_ratio": "16:9",
    "extra": {
      "resolution": "1k"
    }
  }
}
```

如果上游返回 URL，也允许：

```json
{
  "ok": true,
  "command": "task image-gen",
  "data": {
    "provider": "xai",
    "model": "grok-imagine-image",
    "image": "https://...",
    "aspect_ratio": "16:9",
    "extra": {
      "resolution": "1k"
    }
  }
}
```

### `grok-cli task video-gen`

#### 参数

- `--json`
- `--prompt <text>`
- `--image-url <url>` 可选
- `--reference-image-url <url>` 可重复，最多 7 个
- `--duration <seconds>`
- `--aspect-ratio <ratio>`
- `--resolution <resolution>`
- `--model <model>`
- `--auth-file <path>`

#### 成功输出

参考 Hermes 当前实现，视频结果主字段应直接返回远程视频 URL，而不是强制本地下载：

```json
{
  "ok": true,
  "command": "task video-gen",
  "data": {
    "provider": "xai",
    "model": "grok-imagine-video",
    "video": "https://...",
    "modality": "text",
    "aspect_ratio": "16:9",
    "duration": 8,
    "extra": {
      "request_id": "req_123",
      "resolution": "720p"
    }
  }
}
```

### `grok-cli task tts`

#### 参数

- `--json`
- `--text <text>`
- `--voice-id <id>` 可选
- `--language <lang>` 可选
- `--output <path>` 可选
- `--auth-file <path>`

#### 输出路径策略

参考 Hermes 当前真实实现：

- 如果用户传入 `--output`，优先使用该路径
- 否则默认写入：

```text
~/.hermes/cache/audio/audio_cache/
```

#### 成功输出

参考 Hermes 当前字段，主字段应为 `file_path`，而不是重新命名为 `output_path`：

```json
{
  "ok": true,
  "command": "task tts",
  "data": {
    "success": true,
    "provider": "xai",
    "file_path": "/abs/path/audio.mp3",
    "media_tag": "MEDIA:/abs/path/audio.mp3",
    "voice_compatible": false
  }
}
```

### `grok-cli task stt`

#### 参数

- `--json`
- `--file <path>`
- `--model <model>` 可选
- `--language <lang>` 可选
- `--auth-file <path>`

#### 成功输出

参考 Hermes 当前结构：

```json
{
  "ok": true,
  "command": "task stt",
  "data": {
    "success": true,
    "provider": "xai",
    "transcript": "..."
  }
}
```

#### 失败输出

```json
{
  "ok": false,
  "command": "task stt",
  "error": {
    "code": "request_failed",
    "message": "xAI STT transcription failed: ...",
    "relogin_required": false,
    "entitlement_denied": false
  }
}
```

## `proxy` 命令组

### 设计原则

这里直接参考 Hermes，而不是把 proxy 混进 `task`。

因此 `grok-cli proxy` 应设计成：

- `proxy start`
- `proxy status`
- `proxy providers`

如果未来需要 `stop`，也只能在实现里明确新增；当前 spec 不自造 `stop`，因为 Hermes 现状没有该子命令。

### `grok-cli proxy start`

#### 用途

启动本地 OpenAI-compatible 代理，将任意客户端请求转发到 xAI，并自动附加 OAuth bearer。

#### 参数

- `--json`
- `--provider xai`
- `--host <host>` 默认 `127.0.0.1`
- `--port <port>` 默认 `8645`
- `--auth-file <path>`

#### 行为约束

- 前台运行
- 按 Hermes 现状，启动后持续占用终端
- 客户端的 `Authorization` 头会被忽略，由代理附加真实凭据
- 响应体按原样转发，SSE 保留

#### 允许路径

固定为：

- `/v1/responses`
- `/v1/chat/completions`
- `/v1/completions`
- `/v1/embeddings`
- `/v1/models`

其他路径返回：

- HTTP 404
- 结构化错误码：`path_not_allowed`

### `grok-cli proxy status`

#### 用途

展示当前上游适配器状态。

#### 成功输出

```json
{
  "ok": true,
  "command": "proxy status",
  "data": {
    "providers": [
      {
        "name": "xai",
        "display_name": "xAI Grok OAuth",
        "ready": true,
        "authenticated": true,
        "expires_at": "2026-05-19T17:20:00Z"
      }
    ]
  }
}
```

### `grok-cli proxy providers`

#### 用途

列出当前可用上游 provider。

#### 成功输出

```json
{
  "ok": true,
  "command": "proxy providers",
  "data": {
    "providers": [
      {
        "name": "xai",
        "display_name": "xAI Grok OAuth"
      }
    ]
  }
}
```

## `state` 命令组

### `grok-cli state show`

输出状态文件内容摘要。

说明：

- 默认脱敏输出 token

### `grok-cli state path`

输出当前状态文件路径。

### `grok-cli state validate`

校验当前状态文件是否符合 schema。

## `debug` 命令组

### `grok-cli debug authorize-params`

输出当前 authorize URL 相关参数结构，用于调试兼容性。

### `grok-cli debug token-request-shape`

输出 token exchange 请求体结构说明，用于调试兼容性。

### `grok-cli debug hermes-observation`

输出当前实现中与 Hermes 参数观察相关的启用项摘要。

建议至少包含：

- 当前绑定的 `base_url`
- 当前 OAuth provider
- 当前允许路径
- 当前 chat 主协议
- 当前 x_search 默认模型

## 统一错误码建议

建议保留下面这些错误码作为第一批标准错误：

- `invalid_args`
- `io_error`
- `state_file_missing`
- `state_file_invalid`
- `auth_missing`
- `auth_expired`
- `auth_refresh_failed`
- `auth_relogin_required`
- `auth_state_mismatch`
- `auth_callback_timeout`
- `auth_token_exchange_failed`
- `xai_oauth_tier_denied`
- `model_capability_mismatch`
- `request_failed`
- `path_not_allowed`

## 与 SKILL 的配合约定

SKILL 在调用 `grok-cli` 时建议遵循：

1. 先调用 `auth status`
2. 需要时调用 `auth login` 或 `auth refresh`
3. 再调用对应 `task` 命令
4. 如果用户要把 Grok 当成 OpenAI-compatible 端点提供给别的客户端，再进入 `proxy` 命令组
5. 基于 JSON 结果决定用户侧文案

CLI 不负责自然语言解释；CLI 负责把状态和结果稳定交给 SKILL。

## 当前收口结果

这份命令规格已经按 Hermes 当前实现把以下问题定死：

- `task chat --stream`：按 Hermes SSE 事件形状定义
- `proxy`：按 Hermes 真实命令面定义为一级子命令
- `task chat` 的 tool-calling：按 Hermes 的 `finish_reason=tool_calls` 语义定义
- `task x-search`：按 Hermes 当前字段返回 schema 定义
- image / video / tts / stt 输出字段：
  - 图片：`image`
  - 视频：`video`
  - TTS：`file_path`
  - STT：`transcript`

也就是说，这份 spec 已经可以直接作为 `grok-cli` Rust 工程的命令面契约使用。
