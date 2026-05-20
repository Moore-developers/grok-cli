# `grok-cli refresh`

## 用途

使用已保存的 refresh token 强制刷新 access token。

## 常用方式

```bash
grok-cli refresh
```

脚本或 SKILL：

```bash
grok-cli refresh --json
```

## 参数

- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。

## 行为规格

- 读取本地 OAuth 状态文件。
- 使用 refresh token 调用 xAI OAuth token endpoint。
- 成功后原子写回新的 access token、可能更新的 refresh token 和 `last_refresh`。
- 失败时写回 `last_auth_error`，保留 endpoint、phase、grant type、redirect URI 等排障上下文。
- 网络 connect / timeout / request 类错误会做轻量 retry。

## JSON 输出重点

`data` 中包含：

- `provider`
- `refreshed`
- `last_refresh`

## 常见失败

- `auth_missing`：状态文件中没有可用 refresh token。
- `auth_relogin_required`：refresh token 已失效，需要重新 [`login`](./login.md)。
- `auth_token_exchange_failed`：token endpoint 返回失败。

## 相关文档

- [status](./status.md)
- [故障排查](../guides/troubleshooting.md)

