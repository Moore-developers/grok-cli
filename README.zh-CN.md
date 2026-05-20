# grok-cli

一个 OAuth-first 的 Grok / xAI 命令行工具。

[English README](README.md)

`grok-cli` 把 Grok 的常用能力收口成一个本地 CLI：

- 浏览器 OAuth 登录和 token refresh
- Grok 聊天
- 通过 Grok `x_search` 搜索 X
- 图片生成和图片编辑
- 视频生成、视频编辑和视频扩展
- 文本转语音、批量语音转文字和实验性实时语音转文字
- 基于 SQLite 的本地会话 usage 统计

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

从源码安装：

```bash
cargo install --path .
```

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

GitHub Release 二进制包和 Homebrew 发布方式见 [docs/guides/release.md](docs/guides/release.md)。

## 开发

运行测试：

```bash
cargo test
```

构建 release：

```bash
cargo build --release
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
