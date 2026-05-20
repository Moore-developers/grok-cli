# Rust CLI 状态文件 Schema

## 目标

定义 `grok-cli` 的本地状态文件结构，用于保存和复用 Grok OAuth 状态。

这份 schema 的目标是：

- 让 CLI 能稳定读写认证状态
- 让 SKILL 能通过 CLI 间接获得稳定状态判断
- 让未来升级和迁移有版本边界

这里的字段设计仍以独立 CLI 为主，但判断语义会尽量贴近 Hermes 当前对 `xai-oauth` 的状态管理方式。

## 文件路径建议

默认路径建议：

```text
~/.grok-cli/auth.json
```

实现时应允许通过 CLI 参数覆盖。

说明：

- 这里不直接写进 `~/.hermes/auth.json`
- 因为当前目标是做独立 `grok-cli`
- 但 schema 的字段与 Hermes 语义保持尽量接近，方便未来桥接

## 顶层结构

建议顶层采用单对象 JSON：

```json
{
  "version": 1,
  "provider": "xai-oauth",
  "auth_mode": "oauth_pkce",
  "base_url": "https://api.x.ai/v1",
  "tokens": {},
  "discovery": {},
  "redirect_uri": "http://127.0.0.1:56121/callback",
  "last_refresh": "2026-05-19T17:00:00Z",
  "last_auth_error": null,
  "metadata": {}
}
```

## 字段说明

### `version`

- 类型：`integer`
- 必填：是
- 说明：状态文件 schema 版本号

当前建议：

- `1`

### `provider`

- 类型：`string`
- 必填：是

当前固定建议值：

- `xai-oauth`

### `auth_mode`

- 类型：`string`
- 必填：是

当前固定建议值：

- `oauth_pkce`

### `base_url`

- 类型：`string`
- 必填：是

默认值建议：

- `https://api.x.ai/v1`

说明：

- 参考 Hermes 当前 xAI runtime / proxy 语义
- 这里记录的是运行时要访问的 API base URL，而不是 authorize host

### `tokens`

- 类型：`object`
- 必填：是

结构：

```json
{
  "access_token": "string",
  "refresh_token": "string",
  "id_token": "string",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

字段说明：

- `access_token`
  - 类型：`string`
  - 必填：否
  - 说明：当前访问令牌，失效后可能被清空

- `refresh_token`
  - 类型：`string`
  - 必填：否
  - 说明：刷新令牌，若已被判定不可再用，可能被清空

- `id_token`
  - 类型：`string`
  - 必填：否

- `expires_in`
  - 类型：`integer|null`
  - 必填：否

- `token_type`
  - 类型：`string`
  - 必填：否
  - 默认值建议：`Bearer`

补充建议：

- 允许扩展加入 `expires_at`
- 如果实现时写入了 `expires_at`，状态判断应优先使用 `expires_at`
- `expires_in` 更适合作为原始 token exchange 回写值

## `discovery`

- 类型：`object`
- 必填：否

建议结构：

```json
{
  "authorization_endpoint": "https://auth.x.ai/oauth2/authorize",
  "token_endpoint": "https://auth.x.ai/oauth2/token"
}
```

说明：

- 这是 discovery 结果缓存
- 不是绝对可信数据
- 使用前仍应做 host / https 校验

## `redirect_uri`

- 类型：`string`
- 必填：否

建议值示例：

- `http://127.0.0.1:56121/callback`

说明：

- 用于记录当前 OAuth 会话保存时的 `redirect_uri`
- 在 refresh 场景中不是每次都必须使用
- 但对调试和一致性判断有价值

## `last_refresh`

- 类型：`string|null`
- 必填：否

格式建议：

- ISO 8601 UTC 时间戳
- 例如：`2026-05-19T17:00:00Z`

## `last_auth_error`

- 类型：`object|null`
- 必填：否

建议结构：

```json
{
  "provider": "xai-oauth",
  "code": "auth_relogin_required",
  "message": "refresh token 已失效",
  "reason": "runtime_refresh_failure",
  "relogin_required": true,
  "entitlement_denied": false,
  "at": "2026-05-19T17:10:00Z"
}
```

字段说明：

- `provider`
  - 类型：`string`

- `code`
  - 类型：`string`

- `message`
  - 类型：`string`

- `reason`
  - 类型：`string|null`

- `relogin_required`
  - 类型：`boolean`

- `entitlement_denied`
  - 类型：`boolean`

- `at`
  - 类型：`string`
  - ISO 8601 UTC 时间戳

### `last_auth_error` 的语义边界

这里建议明确区分两类失败：

#### 1. 需要重登

例如：

- refresh token 失效
- state 校验失败后旧状态被废弃
- callback 之后 token 无法再续期

此时：

- `relogin_required = true`
- `entitlement_denied = false`

#### 2. entitlement / tier 被拒绝

例如：

- 账号能完成 OAuth
- 但不具备 Grok API 能力使用资格

此时：

- `relogin_required = false`
- `entitlement_denied = true`

这两类一定要分开，因为 SKILL 的用户提示完全不同。

## `metadata`

- 类型：`object`
- 必填：否

用途：

- 保存非核心、但对实现有帮助的扩展信息
- 避免顶层字段不断膨胀

建议字段：

```json
{
  "created_at": "2026-05-19T16:50:00Z",
  "updated_at": "2026-05-19T17:00:00Z",
  "source": "oauth_loopback",
  "cli_version": "0.1.0"
}
```

可选扩展字段：

```json
{
  "oauth_provider_name": "xai",
  "credential_source": "xai-oauth",
  "notes": "reserved"
}
```

## 最小可用状态

为了让 `auth status` 能判断“未登录”，允许存在一个最小状态：

```json
{
  "version": 1,
  "provider": "xai-oauth",
  "auth_mode": "oauth_pkce",
  "base_url": "https://api.x.ai/v1",
  "tokens": {},
  "metadata": {}
}
```

这意味着：

- 文件存在不代表一定已登录
- 是否已登录要看 `tokens.access_token`、可用期、以及错误状态

## 运行时判断建议

虽然磁盘上只存一份 schema，但 Rust 实现里应派生出运行时视图。

建议运行时判断字段：

- `logged_in`
- `access_token_present`
- `refresh_token_present`
- `access_token_expiring`
- `relogin_required`
- `entitlement_denied`

推荐判断逻辑：

- `logged_in = access_token_present || refresh_token_present`
- `relogin_required` 优先看 `last_auth_error.relogin_required`
- `entitlement_denied` 优先看 `last_auth_error.entitlement_denied`
- 如果 token 已不可刷新且 `relogin_required = true`，则 `auth status` 应直接提示重登

## 校验规则

### 必须校验

- `version` 必须存在且可识别
- `provider` 必须为支持值
- `auth_mode` 必须为支持值
- `base_url` 必须是合法 URL
- `tokens` 必须为对象

### 使用前应额外校验

- `discovery.authorization_endpoint` 必须是 `https`
- `discovery.token_endpoint` 必须是 `https`
- discovery host 必须属于 `x.ai` 或 `*.x.ai`

### 推荐校验

- `token_type` 若存在，应允许 `Bearer`
- `redirect_uri` 若存在，应是 loopback 或当前实现允许的显式回调地址
- `last_refresh` / `last_auth_error.at` 应能解析成 UTC 时间

## 状态迁移策略

建议从一开始就保留版本迁移入口。

例如：

- `version = 1`
  当前初始 schema

未来如果要新增字段，应优先保持前向兼容：

- 新字段尽量可选
- 读取旧状态时可自动补默认值

只有在破坏性变更时，才提升主版本号。

## 脱敏规则

凡是输出到：

- `state show`
- 调试输出
- 日志

都应默认脱敏：

- `access_token`
- `refresh_token`
- `id_token`

建议只显示：

- 前 4 位
- 后 4 位

例如：

- `abcd...wxyz`

## 示例：已登录状态

```json
{
  "version": 1,
  "provider": "xai-oauth",
  "auth_mode": "oauth_pkce",
  "base_url": "https://api.x.ai/v1",
  "tokens": {
    "access_token": "access-token",
    "refresh_token": "refresh-token",
    "id_token": "id-token",
    "expires_in": 3600,
    "token_type": "Bearer"
  },
  "discovery": {
    "authorization_endpoint": "https://auth.x.ai/oauth2/authorize",
    "token_endpoint": "https://auth.x.ai/oauth2/token"
  },
  "redirect_uri": "http://127.0.0.1:56121/callback",
  "last_refresh": "2026-05-19T17:00:00Z",
  "last_auth_error": null,
  "metadata": {
    "created_at": "2026-05-19T16:50:00Z",
    "updated_at": "2026-05-19T17:00:00Z",
    "source": "oauth_loopback",
    "cli_version": "0.1.0"
  }
}
```

## 示例：需要重新登录

```json
{
  "version": 1,
  "provider": "xai-oauth",
  "auth_mode": "oauth_pkce",
  "base_url": "https://api.x.ai/v1",
  "tokens": {},
  "last_refresh": "2026-05-19T17:00:00Z",
  "last_auth_error": {
    "provider": "xai-oauth",
    "code": "auth_relogin_required",
    "message": "refresh token 已失效",
    "reason": "runtime_refresh_failure",
    "relogin_required": true,
    "entitlement_denied": false,
    "at": "2026-05-19T17:15:00Z"
  },
  "metadata": {
    "updated_at": "2026-05-19T17:15:00Z"
  }
}
```

## 示例：entitlement 被拒绝

```json
{
  "version": 1,
  "provider": "xai-oauth",
  "auth_mode": "oauth_pkce",
  "base_url": "https://api.x.ai/v1",
  "tokens": {
    "access_token": "access-token",
    "refresh_token": "refresh-token",
    "token_type": "Bearer"
  },
  "last_auth_error": {
    "provider": "xai-oauth",
    "code": "xai_oauth_tier_denied",
    "message": "账号未被允许使用 xAI API OAuth 能力面",
    "reason": "token_exchange_403",
    "relogin_required": false,
    "entitlement_denied": true,
    "at": "2026-05-19T17:20:00Z"
  }
}
```

## 对实现的建议

Rust 中建议把状态拆成两层结构：

- 持久化 schema
- 运行时视图

也就是说：

- 磁盘上按 schema 保存
- 内存中转换成更方便判断的 runtime 状态对象

这样 `auth status`、`task *` 前置认证检查、以及 `proxy status` 的逻辑都会更干净。
