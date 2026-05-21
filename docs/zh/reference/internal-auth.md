# 内部认证救援入口

这份文档记录不在公开 help / README 中展示的认证救援能力。普通用户应优先使用：

```bash
grok-cli login
```

## 已移除的公开入口

### `print-authorize-url`

`print-authorize-url` 已从公开 CLI 入口删除。

删除原因：

- 它主要用于早期验证 OAuth URL 构造，不是普通用户日常需要的动作。
- `grok-cli login --manual-paste` 已经能输出 authorize URL，并且还会正确写入 pending OAuth session。
- 单独暴露该命令会让用户误以为登录流程需要手工拼接多个步骤。

如果需要查看 authorize URL，请使用：

```bash
grok-cli login --manual-paste
```

## 隐藏保留入口

### `exchange-code`

`exchange-code` 仍然保留，但已从公开 help、README 和命令索引中隐藏。

保留原因：

- 它在正常浏览器登录中不需要手动使用。
- 它可以作为救援入口：如果浏览器已经授权成功，但 loopback callback 没命中或 CLI 进程中断，仍可用已有 authorization code 完成 token exchange。

使用方式：

```bash
grok-cli exchange-code --code "AUTHORIZATION_CODE"
```

如果拿到的是完整 callback URL，也可以传给 `--code`：

```bash
grok-cli exchange-code --code "http://127.0.0.1:56121/callback?code=...&state=..."
```

脚本或排障时：

```bash
grok-cli exchange-code --json --code "AUTHORIZATION_CODE"
```

参数：

- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--code <CODE_OR_CALLBACK>`：授权码、完整 callback URL 或 callback query string。必填。
- `--state <STATE>`：显式指定返回的 OAuth state。通常可省略。
- `--redirect-uri <URI>`：覆盖 token exchange 使用的 redirect URI。通常可省略。

行为规格：

- 必须先存在 pending OAuth session。通常由 `grok-cli login --manual-paste` 或一次未完成的 `grok-cli login` 写入。
- 会校验返回的 OAuth state 是否和 pending session 一致。
- 会使用 pending session 中的 PKCE verifier 完成 token exchange。
- 成功后写入 token，清理 pending OAuth session。
- 失败时写回 `last_auth_error`，方便之后 `status` 或 `state` 排障。

成功 JSON 的 `data` 包含：

- `provider`
- `auth_mode`
- `saved`
- `auth_store_path`
- `redirect_uri`
- `base_url`
- `last_refresh`

## 用户路径建议

- 常规登录：`grok-cli login`
- 无法自动回调：`grok-cli login --manual-paste`
- 已有 code 且需要救援：`grok-cli exchange-code --code "..."`
