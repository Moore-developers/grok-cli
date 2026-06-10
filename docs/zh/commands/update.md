# `grok-cli update`

## 用途

检查最新 GitHub Release，并在有新版时升级本地 `grok-cli`。

CLI 也会在面向人类的命令结束后低频检查更新。被动提示写到 stderr，并且会跳过 `--json`、`--raw-stream`、非交互式输出以及 `update` 命令自身。

更新设置保存在：

```text
~/.grok-cli/update.json
```

## 常用方式

只检查，不安装：

```bash
grok-cli update --check
```

检查并在有新版时安装：

```bash
grok-cli update
```

关闭被动更新提示：

```bash
grok-cli update --no-update-check
```

重新开启被动更新提示：

```bash
grok-cli update --enable-update-check
```

脚本或 SKILL 使用：

```bash
grok-cli update --check --json
```

只为单次命令临时关闭被动检查：

```bash
GROK_CLI_NO_UPDATE_CHECK=1 grok-cli chat "你好"
```

## 参数

- `--json`：使用统一 JSON 信封输出。
- `--check`：只检查最新 release，不安装。
- `--force`：即使当前版本已是最新，也重新安装最新 release。
- `--no-update-check`：持久关闭被动后台更新提示。
- `--enable-update-check`：持久重新开启被动后台更新提示。

`--check`、`--force`、`--no-update-check`、`--enable-update-check` 互斥。

## 安装策略

主动升级命令沿用当前安装分发策略：

- macOS Apple Silicon 下载 `grok-cli-macos-aarch64-apple-darwin.tar.gz`，并校验匹配的 `.sha256`。
- Windows x64 下载 `grok-cli-windows-x86_64-pc-windows-msvc.zip`，校验匹配的 `.sha256`，然后启动 PowerShell updater，因为 Windows 通常不能可靠覆盖正在运行的 exe。
- macOS Intel、Linux 和其他 source-first 平台执行 `cargo install --git https://github.com/Moore-developers/grok-cli.git --tag <LATEST_TAG> --locked --force`。

如果所需 release asset 缺失或 checksum 校验失败，命令会返回结构化错误，不会静默切换安装策略。

## JSON 输出重点

`update --check --json` 返回：

- `current_version`
- `latest_version`
- `latest_tag`
- `update_available`
- `release_url`
- `install_strategy`
- `asset_name`

`update --json` 还会返回：

- `installed`
- `message`

`update --no-update-check --json` 和 `update --enable-update-check --json` 返回：

- `auto_check_enabled`
- `update_config_path`

## 相关文档

- [发布与安装指南](../guides/release.md)
- [故障排查](../guides/troubleshooting.md)
