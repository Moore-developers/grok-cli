# 验收样例

这份文档给出一组可复现的最小验收路径，便于在接入 SKILL、交付或回归时快速确认能力是否完整。

## 1. 状态与认证

### 验收 1：状态摘要

```bash
cargo run -- state --json
```

期望：
- 返回 `ok: true`
- 返回 `command: "state"`
- 输出脱敏状态摘要或 `exists: false`

### 验收 2：浏览器登录

```bash
cargo run -- login --json
```

期望：
- 能拉起真实浏览器
- 登录成功后状态可落盘

### 验收 3：刷新

```bash
cargo run -- refresh --json
```

期望：
- 成功时刷新 `last_refresh`

## 2. 任务能力

### 验收 4：文本默认模型切换

```bash
cargo run -- model --json
cargo run -- model --json --model grok-4.3
```

期望：
- `model --json` 列出共享文本模型目录
- `model --model ...` 能同时写入 `chat` 与 `search` 默认模型
- `grok-cli mode` 作为 `model` 别名可用

### 验收 5：聊天

```bash
cargo run -- chat "用一句话介绍 Grok"
cargo run -- chat "总结最近 AI 新闻" --with-x-search
```

期望：
- 返回 `protocol: "codex_responses"`
- 返回 `output_text`
- 默认以格式化流式正文输出
- 默认请求体会附带通用 `web_search`
- 指定 `--with-x-search` 时，请求体会同时附带 `web_search + x_search`
- `--json` 时返回稳定的单次结构化结果

### 验收 6：X 搜索

```bash
cargo run -- search "What are people saying about xAI on X today?"
```

期望：
- 返回 `answer`
- 返回 `citations`
- 默认以格式化流式正文输出
- `--json` 时返回稳定的单次结构化结果

### 验收 7：媒体能力

分别执行：

```bash
cargo run -- image "A cinematic skyline"
cargo run -- image "A cinematic skyline" --count 2 --response-format url --json
cargo run -- image "A logo mark" --count 2 --output-dir ./out/images --json
cargo run -- video "Animate a cinematic skyline" --duration 8
cargo run -- tts "Hello from Grok"
cargo run -- tts "Hello from Grok" --output-format mp3 --sample-rate 24000 --bit-rate 128000
cargo run -- tts --list-voices --json
cargo run -- stt ./sample.wav
cargo run -- stt ./sample.wav --diarize --keyterm Grok --filler-words --json
```

期望：
- 图片返回 `image`
- 图片多图请求返回 `image` 和 `images`
- 图片多图 base64 落盘请求返回本地路径列表
- 视频返回 `video`
- TTS 返回 `file_path`
- TTS 显式输出格式请求成功时返回 `output_format`
- `tts --list-voices --json` 返回 `voices`
- STT 返回 `transcript`
- STT 高级参数请求成功时仍保留 `transcript`，并在上游返回时保留 `language` / `duration` / `words` / `channels`
- 当 access token 接近过期时，媒体命令会先自动 refresh，再发起真实请求

真实验收补充：

- `2026-05-20` 已完成真实 `image`
- `2026-05-20` 已完成真实 `tts -> stt` 串联验证
- `2026-05-20` 已完成真实 `video` 三条分支验证：
  - text-to-video
  - image-to-video
  - reference-image video
- `2026-05-20` 已确认媒体命令在 access token 临近过期时会先自动 refresh，再继续真实请求，不再要求用户手动先执行 `refresh`

### 验收 8：Usage

```bash
cargo run -- usage
cargo run -- usage --json
```

期望：
- `--json` 返回 `session` / `local_usage` / `breakdown` / `recent_rate_limits`
- `breakdown` 至少覆盖 `text` / `image` / `video` / `audio`
- 人类输出按 `Session Usage` / `Usage Breakdown` 分组展示
- token 数字按 `K/M/B` 紧凑格式展示
- 人类输出和 JSON 都不包含 Account limits

## 3. 回归验收

执行：

```bash
cargo test
```

期望：
- 所有单元测试、命令级测试、契约回归测试通过
