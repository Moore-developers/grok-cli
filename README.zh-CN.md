# grok-cli

## 概览

`grok-cli` 把 Grok / xAI 带进终端优先、脚本优先和 agent 驱动的工作流。它支持通过 SuperGrok 或 X Premium+ 订阅直接 OAuth 登录，不需要额外 API Key 或单独付费体系。

它给你一套 CLI，统一登录、聊天、搜索、媒体、音频和 usage，也把认证、自动化输出、本地文件、远程 URL 和不同平台安装收进同一个入口。

OpenClaw 和 Hermes Agent 覆盖官方支持的集成路径；`grok-cli` 适合 Codex、Claude Code、Cursor、自定义自动化、agent runtime、skill、脚本、CI 和验证流程。

## 功能介绍

- 文本：带网络搜索的 Grok 聊天，以及 X 搜索。
- 媒体：图片生成和编辑，以及视频生成、编辑和扩展。
- 音频：文本转语音和语音转文字。

## 快速安装

先选一种最适合你的方式：

- **Skill**：适合 Codex、Claude Code、Cursor 和其他 agent runtime。它会把内置 skill 安装到你的 agent 环境里，让助手自己处理安装检查、OAuth 登录和命令路由。

  ```bash
  npx --yes skills add https://github.com/Moore-developers/grok-cli --skill grok-cli --global --yes
  ```

- **Cargo**：适合从源码构建，或者你在 macOS Intel 和 Linux 上使用。这条路径会直接从仓库安装 CLI，适合保留完整源码工作流。

  ```bash
  cargo install --git https://github.com/Moore-developers/grok-cli.git --locked
  ```

- **Release binary**：适合 macOS Apple Silicon 或 Windows，想跳过 Rust 安装时使用。从 GitHub Releases 下载对应产物后解压即可。

  从 [GitHub Releases](https://github.com/Moore-developers/grok-cli/releases/latest) 下载。

如果你不确定，agent 工作流优先选 Skill，源码构建优先选 Cargo。

文本命令同时照顾“给人直接用”和“给脚本稳定接入”两种场景：

- `chat` 和 `search` 默认以流式方式打印可读正文，适合人直接盯着看
- `--json` 会切回稳定的非流式结构化结果，适合 SKILL、脚本和自动化
- `--stream` 是显式声明“使用格式化流式输出”
- `--raw-stream` 才会输出原始事件流，适合调试或程序消费

公开命令面是扁平的：

```text
grok-cli <login|status|refresh|logout|state|model|usage|chat|search|image|image-edit|video|video-edit|video-extend|tts|stt|stt-stream> ...
```

## 快速开始

浏览器登录：

```bash
grok-cli login
```

查看登录状态：

```bash
grok-cli status
```

聊天：

```bash
grok-cli chat "总结最近 AI 新闻"
```

搜索 X：

```bash
grok-cli search "今天大家怎么评价 Grok?"
```

生成媒体：

```bash
grok-cli image "日出时分的电影感城市天际线"
grok-cli image-edit --image ./source.png --prompt "变得更有电影感"
grok-cli video "让未来城市天际线动起来" --duration 8
grok-cli video-edit --video-url https://example.com/source.mp4 --prompt "变得更有电影感"
grok-cli video-extend --video-url https://example.com/source.mp4 --prompt "延续镜头运动" --duration 6
grok-cli tts "你好，我是 Grok"
grok-cli stt ./sample.wav
grok-cli stt-stream ./sample.wav --interim-results
```

查看本地用量：

```bash
grok-cli usage
```

## 脚本模式

给人用时，推荐直接写位置参数。给脚本、SKILL 或自动化用时，可以继续使用显式参数和 JSON：

```bash
grok-cli chat --json --prompt "总结最近 AI 新闻"
grok-cli search --json --query "Grok Hermes 最新动态"
grok-cli image --json --prompt "一座赛博朋克城市"
grok-cli image-edit --json --image ./source.png --prompt "变得更有电影感"
grok-cli tts --json --text "你好，我是 Grok"
grok-cli stt --json --file ./sample.wav
grok-cli stt-stream --json --file ./sample.wav
grok-cli usage --json
```

如果你想要人类可读但不想看流式过程，可以显式加 `--no-stream`：

```bash
grok-cli chat "总结最近 AI 新闻" --no-stream
grok-cli search "今天大家怎么评价 Grok?" --no-stream
```

成功 JSON 统一长这样：

```json
{
  "ok": true,
  "command": "chat",
  "data": {}
}
```

失败 JSON 也使用统一结构：

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

## 命令说明

- `login`：用系统浏览器发起 xAI OAuth 登录。
- `status`：查看当前是否有可用 OAuth 会话。
- `refresh`：刷新已保存的 access token。
- `logout`：删除本地登录状态。
- `chat`：执行 Grok 文本聊天，默认带通用网页搜索。
- `search`：通过 Grok `x_search` 搜索 X。
- `image`：生成图片。
- `image-edit`：基于一张或多张参考图编辑图片。
- `video`：生成视频。
- `video-edit`：编辑已有视频。
- `video-extend`：扩展已有视频。
- `tts`：文本转语音。
- `stt`：语音转文字。
- `stt-stream`：通过 WebSocket 实时语音转文字，当前是实验入口。
- `usage`：查看本地 session usage 和最近 rate-limit 快照。
- `model`：配置 `chat` 和 `search` 共享默认文本模型。
- `state`：查看本地认证状态的脱敏摘要。

任何命令都可以加 `--help`：

```bash
grok-cli chat --help
grok-cli usage --help
```

## 状态文件

默认路径：

- OAuth 状态：`~/.grok-cli/auth.json`
- Session usage 数据库：`~/.grok-cli/session.db`

OAuth token 存在 `auth.json`。

Usage 历史存在 SQLite，包含 session 总量、每次命令事件、文本/图片/视频/音频分类统计，以及最近一次 rate-limit 快照。

媒体文件内容不会存进 SQLite。

## 安装方式

从源码安装：

```bash
git clone https://github.com/Moore-developers/grok-cli.git
cd grok-cli
cargo install --path .
```

GitHub 仓库公开后，可以直接安装：

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked
```

安装指定 tag：

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked
```

已覆盖的 Release 产物：

- macOS Apple Silicon：`grok-cli-macos-aarch64-apple-darwin.tar.gz`
- Windows x64：`grok-cli-windows-x86_64-pc-windows-msvc.zip`

每个发布产物都应该有一个同名 `.sha256` 校验文件。预构建二进制包不是完整平台矩阵，而是按当前可维护的平台提供。推荐使用 `cargo install --git`，或者通过内置 [`grok-cli` skill](skills/grok-cli/SKILL.md) 自动完成安装和命令调用。

## 开发

欢迎参与贡献。提交 PR 前请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)，安全问题请按 [SECURITY.md](SECURITY.md) 私下报告。

运行测试：

```bash
cargo test
```

构建 release：

```bash
cargo build --release
```

打包并上传本地 macOS Apple Silicon 发布产物：

```bash
scripts/package-local-macos-release.sh v0.1.0 --upload
```

安装本地版本：

```bash
cargo install --path . --force
```

## 文档

- [English README](README.md)
- [文档索引](docs/index.md)
- [快速开始](docs/guides/quickstart.md)
- [命令参考](docs/commands/index.md)
- [`usage` 命令规格](docs/reference/usage-command-spec.md)
- [发布与安装指南](docs/guides/release.md)
- [故障排查](docs/guides/troubleshooting.md)
- [更新日志](CHANGELOG.md)
- [贡献指南](CONTRIBUTING.md)
- [安全策略](SECURITY.md)
