# `grok-cli login`

## 用途

启动 xAI OAuth PKCE 登录。默认会打开系统浏览器，并在本地 loopback callback 收到授权码后自动换取 token，写入本地认证状态。

默认状态文件：

```text
~/.grok-cli/auth.json
```

## 常用方式

```bash
grok-cli login
```

脚本或 SKILL 推荐 JSON：

```bash
grok-cli login --json
```

如果浏览器无法自动回调本地 callback，可以启用手工回贴：

```bash
grok-cli login --manual-paste
```

只打印登录流程状态并自定义端口：

```bash
grok-cli login --port 56121 --timeout 180
```

## 参数

- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。
- `--no-browser`：不自动打开浏览器，只准备本地登录状态。
- `--manual-paste`：使用手工回贴 callback / authorization code 模式。
- `--timeout <SECONDS>`：等待 loopback callback 的超时时间。
- `--port <PORT>`：指定本地 callback 端口。

## 行为规格

- 生成 PKCE verifier / challenge、OAuth `state` 和 `nonce`。
- 构造带 `referrer=hermes-agent` 与 `plan=generic` 的 xAI 授权 URL。
- 将 pending OAuth session 写入 `auth.json`。
- 默认打开真实系统浏览器。
- loopback callback 成功时，自动执行 authorization code exchange。
- callback 超时时，在交互式终端中自动降级到 manual-paste fallback。
- 成功后写入 access token、refresh token、redirect URI、last refresh 信息。

## JSON 输出

成功时：

```json
{
  "ok": true,
  "command": "login",
  "data": {
    "provider": "xai-oauth",
    "auth_mode": "oauth_pkce",
    "saved": true,
    "auth_store_path": "~/.grok-cli/auth.json",
    "redirect_uri": "http://127.0.0.1:56121/callback",
    "base_url": "https://api.x.ai/v1"
  }
}
```

失败时遵循统一错误信封，常见错误包括 `auth_callback_timeout`、`auth_state_mismatch`、`auth_token_exchange_failed`。

## 相关文档

- [status](./status.md)
- [内部认证救援入口](../reference/internal-auth.md)
- [故障排查](../guides/troubleshooting.md)
