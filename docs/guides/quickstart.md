# 快速开始

## 1. 编译与检查

在项目根目录运行：

```bash
cargo test
```

如果测试全绿，说明本地构建、认证状态处理、任务执行和 usage 主路径都已经可用。

## 2. 查看当前认证状态

```bash
cargo run -- status --json
```

如果还没有认证状态文件，典型返回会包含：

```json
{
  "ok": false,
  "command": "status",
  "error": {
    "code": "state_file_missing",
    "message": "state file not found: ...",
    "relogin_required": false,
    "entitlement_denied": false
  }
}
```

## 3. 发起浏览器登录

```bash
cargo run -- login --json
```

如果你在无浏览器环境中执行，或者要用手工回贴模式，可用：

```bash
cargo run -- login --json --manual-paste
```

说明：
- 给人使用时可以不加 `--json`
- 给脚本、SKILL 或测试使用时建议加 `--json`

## 4. 执行第一个真实任务

### 可选：先设置文本默认模型

```bash
cargo run -- model --json
cargo run -- model --json --model grok-4.3
```

说明：
- `model` 当前只管理共享文本默认模型：`chat` 和 `search` 使用同一个选择
- 交互式终端中也可以运行 `cargo run -- model`，用方向键选择模型
- 图片、视频、TTS、STT 如需切模型，继续直接在命令上显式传 `--model`

### 聊天

```bash
cargo run -- chat "用一句话介绍 Grok"
cargo run -- chat "总结最近 AI 新闻" --with-x-search
```

说明：
- `chat` 默认会自动带通用 `web_search`
- `chat` 默认会流式打印可读正文
- 如果你只想要纯聊天，可加 `--no-web-search`
- 如果你想关闭默认流式，可加 `--no-stream`
- 如果既要通用网页搜索，又要 X 动态搜索，可加 `--with-x-search`

### X 搜索

```bash
cargo run -- search "What are people saying about xAI on X today?"
```

说明：
- `search` 默认也会流式打印可读正文
- 如果你想拿单次最终文本，可加 `--no-stream`

### 图片生成

```bash
cargo run -- image "A cinematic skyline at sunrise"
```

### 图片编辑

```bash
cargo run -- image-edit --image ./source.png --prompt "Make it cinematic"
```

### 视频生成

```bash
cargo run -- video "Animate a futuristic skyline" --duration 8
```

### 视频编辑

```bash
cargo run -- video-edit --video-url https://example.com/source.mp4 --prompt "Make it cinematic"
```

### 文本转语音

```bash
cargo run -- tts "Hello from Grok"
```

### 语音转文字

```bash
cargo run -- stt ./sample.wav
```

### 实时语音转文字

```bash
cargo run -- stt-stream ./sample.wav --interim-results
```

说明：
- 图片、图片编辑、视频、视频编辑、TTS、STT 现在都会在发请求前检查 access token 是否即将过期
- 如果已接近过期，命令会先自动 refresh，再继续真实请求
- `stt-stream` 是实验入口，适合继续验证 WebSocket 实时转写协议

## 5. 查看会话使用量

```bash
cargo run -- usage
```

说明：
- `usage` 会先输出本地 session usage
- `usage` 不查询、不展示、不返回 Account limits

## 6. 继续阅读

- 命令细节见 [命令参考](../commands/index.md)
- 自动化调用约定见 [SKILL 集成约定](../reference/skill-integration.md)
- 样例 JSON 见 [示例状态与样例输出](../reference/samples.md)
- 常见错误看 [故障排查](./troubleshooting.md)
