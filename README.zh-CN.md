# grok-cli

> 把 Grok / xAI 带进终端、脚本和 agent 工作流的一把 CLI。

## 特性

- **OAuth 认证** — SuperGrok 或 X Premium+ 登录，无需 API Key。
- **扁平命令面** — 一个 CLI 搞定聊天、搜索、图片、视频、音频和 usage。
- **默认流式输出** — 人类可读正文，`--json` 给自动化用。
- **媒体输入** — 图片、视频、音频支持本地文件和远程 URL。
- **跨平台** — macOS Apple Silicon 和 Windows x64 提供预构建包。

## 安装

```bash
# Agent runtime（推荐）
npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes

# 从源码安装（需 Rust 1.88+）
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked

# 预编译包
# 从 GitHub Releases 下载
```

源码安装需 Rust 1.88+（工具链锁定 1.92.0）。macOS Apple Silicon 和 Windows x64 上内置 skill 优先使用 release binary。

预编译包名称：

- macOS Apple Silicon：`grok-cli-macos-aarch64-apple-darwin.tar.gz`
- Windows x64：`grok-cli-windows-x86_64-pc-windows-msvc.zip`

安装后可以检查 CLI 更新：

```bash
grok-cli update --check
```

有新版时运行 `grok-cli update` 升级。被动更新提示是低频的，并且不会出现在 `--json` 或 raw stream 输出中。可用 `grok-cli update --no-update-check` 关闭，用 `grok-cli update --enable-update-check` 恢复，或用 `GROK_CLI_NO_UPDATE_CHECK=1` 只临时关闭单次命令的被动检查。

## 快速开始

```bash
# 1. 浏览器登录
grok-cli login

# 2. 查看登录状态
grok-cli status

# 3. 第一条命令
grok-cli chat "总结最近 AI 新闻"

# 4. 查看用量
grok-cli usage
```

没有浏览器？用 `grok-cli login --manual-paste` 走验证码登录。

## 使用方式

### 聊天与搜索

```bash
# 流式输出（默认）
grok-cli chat "AI 领域有什么新进展？"
grok-cli search "大家怎么评价 Grok 3？"

# 非流式
grok-cli chat "总结 AI 新闻" --no-stream

# 带 X 搜索结果的聊天
grok-cli chat "xAI 最新动态" --with-x-search

# 纯聊天不带网页搜索
grok-cli chat "你好" --no-web-search

# JSON 给脚本用
grok-cli search --json --query "Grok 更新"
```

### 图片与视频

```bash
grok-cli image "日出时分的电影感城市天际线"
grok-cli image-edit --image ./source.png --prompt "变得更有电影感"
grok-cli video "让未来城市天际线动起来" --duration 8
grok-cli video-edit --video-url https://example.com/source.mp4 --prompt "变得更有电影感"
grok-cli video-extend --video-url https://example.com/source.mp4 --prompt "延续镜头运动" --duration 6
```

### 音频

```bash
grok-cli tts "你好，我是 Grok"
grok-cli stt ./sample.wav
grok-cli stt-stream ./sample.wav --interim-results
```

### 模型

```bash
# 查看当前模型
grok-cli model

# 设置 chat 和 search 的默认模型
grok-cli model --model grok-4.3
```

## JSON 输出

所有命令都支持 `--json`，输出结构统一：

```json
{
  "ok": true,
  "command": "chat",
  "data": {}
}
```

失败时：

```json
{
  "ok": false,
  "command": "chat",
  "error": {
    "code": "auth_missing",
    "message": "...",
    "relogin_required": false,
    "entitlement_denied": false
  }
}
```

## 给 AI agent 用

适合 Codex、Claude Code、Cursor 等 agent runtime。安装内置 skill，自动处理认证和命令路由：

```bash
npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes
```

## 命令说明

| 命令 | 说明 |
| --- | --- |
| `login` | 在系统浏览器中发起 xAI OAuth 登录 |
| `status` | 检查 OAuth 会话状态 |
| `refresh` | 刷新 access token |
| `logout` | 删除本地登录状态 |
| `chat` | Grok 文本聊天（默认含网页搜索） |
| `search` | 搜索 X |
| `image` | 生成图片 |
| `image-edit` | 编辑参考图片 |
| `video` | 生成视频 |
| `video-edit` | 编辑视频 |
| `video-extend` | 扩展视频 |
| `tts` | 文字转语音 |
| `stt` | 语音转文字 |
| `stt-stream` | WebSocket 实时语音转文字（实验） |
| `usage` | 查看本地用量和 rate-limit 快照 |
| `update` | 检查更新、升级 CLI，并管理被动更新提示 |
| `model` | 设置 `chat` 和 `search` 的默认模型 |
| `state` | 查看脱敏的本地认证状态 |

任何命令加 `--help` 查看详情。

## 状态文件

- **Auth token**：`~/.grok-cli/auth.json`
- **Usage 历史**：`~/.grok-cli/session.db`（SQLite）
- **更新设置**：`~/.grok-cli/update.json`

Usage 记录 session 总量、每次命令事件、媒体类型统计和 rate-limit 快照。媒体文件不存数据库。

## 开发

```bash
cargo test
cargo build --release
cargo install --path . --force
```

欢迎贡献，请阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 和 [SECURITY.md](SECURITY.md)。

## 文档

- [快速开始](docs/zh/guides/quickstart.md)
- [命令参考](docs/zh/commands/index.md)
- [故障排查](docs/zh/guides/troubleshooting.md)
- [更新日志](CHANGELOG.md)
