# `grok-cli model`

## 用途

查看并选择文本命令的共享默认模型。`chat` 和 `search` 使用同一个默认模型；切换一次会同时影响两者。

`grok-cli mode` 是 `grok-cli model` 的别名。

## 常用方式

交互选择：

```bash
grok-cli model
grok-cli mode
```

在交互式终端中会显示以下列表，并支持方向键上下选择，回车确认：

```text
grok-4.3
grok-4.20-reasoning
grok-4.20-0309-reasoning
exit
```

选中模型后会提示：

```text
Model switched to <MODEL>.
```

脚本或 SKILL 查看：

```bash
grok-cli model --json
```

脚本或 SKILL 直接设置：

```bash
grok-cli model --json --model grok-4.3
```

## 参数

- `--json`：使用统一 JSON 信封输出；不会进入交互选择。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--model <MODEL>`：保存为 `chat` 和 `search` 的共享默认模型。

## 行为规格

- `grok-cli model` 需要可读取的 auth state。
- 不再提供 `show`、`list`、`set` 子命令。
- 未传 `--model` 时，交互终端进入方向键选择；非交互环境输出当前选择和模型目录。
- 传入 `--model` 时，将共享文本模型写入 `auth.json` 的 metadata。
- `--command` / `--task` 仅作为隐藏兼容参数保留；公开行为永远是 chat/search 共用同一个模型。
- 媒体命令如果要指定模型，请直接在 `image`、`image-edit`、`video`、`video-edit`、`tts`、`stt`、`stt-stream` 上传 `--model`。

## JSON 输出重点

查看模式的 `data` 包含：

- `provider`
- `selected_model`
- `selected`
- `catalog`

设置模式的 `data` 包含：

- `provider`
- `model`
- `selected`
- `catalog`

## 相关文档

- [chat](./chat.md)
- [search](./search.md)
