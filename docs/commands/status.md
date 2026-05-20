# `grok-cli status`

## 用途

查看当前 OAuth 状态是否可用。它不会发起网络请求，只读取本地 `auth.json` 并判断 token、refresh token、过期状态和错误标记。

## 常用方式

```bash
grok-cli status
```

脚本或 SKILL：

```bash
grok-cli status --json
```

检查指定状态文件：

```bash
grok-cli status --auth-file ./auth.json --json
```

## 参数

- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。

## 行为规格

- 读取并校验 OAuth 状态文件。
- 非 JSON 输出使用三列表格：字段、当前值、英文说明。
- 输出 `logged_in`、`access_token_present`、`refresh_token_present`、`access_token_expiring`。
- 输出 `relogin_required` 和 `entitlement_denied`，便于上层判断是需要重登还是账号权限不足。
- 不自动 refresh。需要刷新时请使用 [`refresh`](./refresh.md)。

## JSON 输出重点

`data` 中包含：

- `logged_in`
- `provider`
- `auth_mode`
- `access_token_present`
- `refresh_token_present`
- `access_token_expiring`
- `relogin_required`
- `entitlement_denied`
- `auth_store_path`
- `base_url`
- `last_refresh`
- `last_auth_error`

## 相关文档

- [login](./login.md)
- [refresh](./refresh.md)
- [state](./state.md)
