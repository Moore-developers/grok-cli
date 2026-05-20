# 示例状态与样例输出

## 1. 示例状态文件

示例文件位置：

[`./.sample/auth.json`](../../.sample/auth.json)

用途：
- 说明状态文件 schema 的基本形状
- 说明 `provider`、`auth_mode`、`discovery`、`redirect_uri`、`metadata` 的位置

注意：
- 这是示例值
- 不能直接用于真实认证

## 2. `status --json`

```json
{
  "ok": true,
  "command": "status",
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

## 3. `chat --json`

```json
{
  "ok": true,
  "command": "chat",
  "data": {
    "provider": "xai-oauth",
    "model": "grok-4.3",
    "protocol": "codex_responses",
    "output_text": "hello",
    "finish_reason": "stop",
    "tool_calls": []
  }
}
```

默认行为说明：
- `chat` 默认会挂载 `web_search`
- `chat` 非 `--json` 时默认流式打印可读正文
- `chat --json` 默认返回单次结构化结果
- `chat --no-web-search` 会退回纯聊天
- `chat --with-x-search` 会同时挂载 `web_search + x_search`

## 3.1 `model --json`

```json
{
  "ok": true,
  "command": "model",
  "data": {
    "provider": "xai-oauth",
    "selected_model": "grok-4.3",
    "selected": {
      "text": "grok-4.3",
      "chat": "grok-4.3",
      "search": "grok-4.3"
    },
    "catalog": [
      "grok-4.3",
      "grok-4.20-reasoning",
      "grok-4.20-0309-reasoning"
    ]
  }
}
```

补充说明：

- `selected` 展示共享文本模型，以及兼容用的 `chat` / `search` 展开值
- `image`、`video`、`tts`、`stt` 不在这里持久化默认值

## 4. `search --json`

```json
{
  "ok": true,
  "command": "search",
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

默认行为说明：
- `search` 非 `--json` 时默认流式打印可读正文
- `search --json` 默认返回单次结构化结果

## 5. `image --json`

```json
{
  "ok": true,
  "command": "image",
  "data": {
    "provider": "xai",
    "credential_source": "xai-oauth",
    "model": "grok-imagine-image",
    "image": "https://cdn.x.ai/image-1.png",
    "images": [
      "https://cdn.x.ai/image-1.png",
      "https://cdn.x.ai/image-2.png"
    ],
    "aspect_ratio": "16:9",
    "extra": {
      "resolution": "1k",
      "count": 2,
      "response_format": "url"
    }
  }
}
```

兼容性说明：
- `image` 始终是第一张图片，旧脚本可以继续读取它。
- `images` 是完整图片列表，单图时也会返回一个元素。
- 使用 `--output-dir` 时，`image` 和 `images` 返回本地落盘路径。

## 6. `tts --json`

```json
{
  "ok": true,
  "command": "tts",
  "data": {
    "success": true,
    "provider": "xai",
    "credential_source": "xai-oauth",
    "file_path": "/abs/path/audio.mp3",
    "media_tag": "MEDIA:/abs/path/audio.mp3",
    "voice_compatible": false
  }
}
```

## 6.1 媒体能力真实验证补充

`2026-05-20` 真实验证结果：

- `image --json` 已成功返回真实 xAI CDN 图片链接
- `tts --json` 已成功生成真实 MP3 文件
- `stt --json` 已成功转写上一条真实 TTS 生成的 MP3 文件
- `video --json` 已成功完成以下三条真实分支：
  - text-to-video
  - image-to-video（`--image-url`）
  - reference-image video（`--reference-image-url`）

真实验证期间发现并已吸收的接口约束：

- `stt` 当请求里带 `format=true` 时，xAI 要求必须同时提供 `language`
- `video` 在 access token 接近过期时，视频接口比聊天接口更容易直接返回 token validation failure
- 当前媒体请求入口已接入“access token 即将过期则先 refresh”的自动编排

这轮真实问题定位结论：

- 首次失败报文为 `The OAuth2 access token could not be validated. [WKE=unauthenticated:bad-credentials]`
- 随后检查 `status`，可见当前状态已处于 `access_token_expiring: true`
- 手动执行一次 `refresh` 后，`video` 的三条真实分支全部恢复成功
- 现已把这一步前置为媒体命令内建编排，不再要求用户自己先发现再刷新

## 7. 典型错误信封

```json
{
  "ok": false,
  "command": "status",
  "error": {
    "code": "state_file_missing",
    "message": "state file not found: /abs/path/auth.json",
    "relogin_required": false,
    "entitlement_denied": false
  }
}
```

## 8. `usage`

人类可读输出示例：

```text
Session Usage
├─ Input tokens:     125K
├─ Output tokens:    45.3K
├─ Total tokens:     170K
├─ Estimated cost:   $0.27 (this session)
├─ Duration:         47m 12s
└─ Context:          170K / 2.00M tokens (8.5%)

Usage Breakdown
├─ Text
│   ├─ Requests:       5
│   ├─ Commands:       chat, search
│   ├─ Input tokens:   125K
│   ├─ Output tokens:  45.3K
│   ├─ Total tokens:   170K
│   ├─ Reasoning:      3.00K
│   └─ Estimated cost: $0.27
├─ Image
│   ├─ Requests:       1
│   ├─ Commands:       image
│   ├─ Input tokens:   0
│   ├─ Output tokens:  0
│   ├─ Total tokens:   0
│   ├─ Reasoning:      0
│   └─ Estimated cost: n/a
├─ Video
│   ├─ Requests:       1
│   ├─ Commands:       video
│   ├─ Input tokens:   0
│   ├─ Output tokens:  0
│   ├─ Total tokens:   0
│   ├─ Reasoning:      0
│   └─ Estimated cost: n/a
├─ Audio
│   ├─ Requests:       2
│   ├─ Commands:       tts, stt
│   ├─ Input tokens:   3.20K
│   ├─ Output tokens:  600
│   ├─ Total tokens:   3.80K
│   ├─ Reasoning:      0
│   └─ Estimated cost: $0.00

```

`--json` 输出示例：

```json
{
  "ok": true,
  "command": "usage",
  "data": {
    "provider": "xai-oauth",
    "session": {
      "session_id": "sess_01_example",
      "started_at": "2026-05-20T10:00:00Z",
      "last_activity_at": "2026-05-20T10:47:12Z",
      "duration_seconds": 2832,
      "request_count": 7,
      "tracked_command_count": 7,
      "models": ["grok-4.20-reasoning"],
      "session_store_path": "/abs/path/session.db"
    },
    "local_usage": {
      "input_tokens": 124837,
      "output_tokens": 45291,
      "cache_read_tokens": 0,
      "cache_write_tokens": 0,
      "reasoning_tokens": 0,
      "total_tokens": 170128,
      "estimated_cost_usd": 0.269193,
      "pricing_status": "estimated",
      "pricing_source": "bundled_xai_table",
      "last_model": "grok-4.20-reasoning",
      "context_window_tokens": 2000000,
      "history_turns": 7,
      "compression_count": 0,
      "has_unflushed_tracker_data": false
    },
    "breakdown": {
      "text": {
        "request_count": 5,
        "commands": ["chat", "search"],
        "input_tokens": 124837,
        "output_tokens": 45291,
        "cache_read_tokens": 0,
        "cache_write_tokens": 0,
        "reasoning_tokens": 3000,
        "estimated_cost_usd": 0.269193
      },
      "image": {
        "request_count": 1,
        "commands": ["image"],
        "input_tokens": 0,
        "output_tokens": 0,
        "cache_read_tokens": 0,
        "cache_write_tokens": 0,
        "reasoning_tokens": 0,
        "estimated_cost_usd": 0.0
      },
      "video": {
        "request_count": 1,
        "commands": ["video"],
        "input_tokens": 0,
        "output_tokens": 0,
        "cache_read_tokens": 0,
        "cache_write_tokens": 0,
        "reasoning_tokens": 0,
        "estimated_cost_usd": 0.0
      },
      "audio": {
        "request_count": 2,
        "commands": ["tts", "stt"],
        "input_tokens": 3200,
        "output_tokens": 600,
        "cache_read_tokens": 0,
        "cache_write_tokens": 0,
        "reasoning_tokens": 0,
        "estimated_cost_usd": 0.0
      }
    },
    "recent_rate_limits": {
      "available": true
    }
  }
}
```

补充说明：

- `usage` 人类可读输出只展示本地 Session Usage 和 Usage Breakdown
- `usage` 现已按 `text` / `image` / `video` / `audio` 四类输出 breakdown
- 非 `--json` 输出中的 token 数字统一按 `K/M/B` 紧凑格式展示
- `usage` 不查询、不展示、不返回 Account limits
