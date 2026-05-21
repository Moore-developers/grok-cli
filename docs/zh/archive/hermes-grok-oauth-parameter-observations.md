# Hermes Grok OAuth 参数观察清单

## 目的

这份文档用于整理 Hermes 当前在使用 Grok OAuth 时涉及到的关键参数、固定值、
请求形状和行为特征。

这些内容在未来的 Grok SKILL 设计里应被视为：

- 兼容性观察
- 行为参考
- 风险输入

而不是直接暴露给最终用户的表层概念。

## 关键常量

根据 Hermes 当前源码，可以观察到以下关键值：

### OAuth issuer 与 discovery

- `XAI_OAUTH_ISSUER = https://auth.x.ai`
- `XAI_OAUTH_DISCOVERY_URL = https://auth.x.ai/.well-known/openid-configuration`

### OAuth client

- `XAI_OAUTH_CLIENT_ID = b1a00492-073a-47ea-816f-4c329264a828`

### OAuth scope

- `XAI_OAUTH_SCOPE = openid profile email offline_access grok-cli:access api:access`

### 回调地址

- `XAI_OAUTH_REDIRECT_HOST = 127.0.0.1`
- `XAI_OAUTH_REDIRECT_PORT = 56121`
- `XAI_OAUTH_REDIRECT_PATH = /callback`

### 默认 API base

- `DEFAULT_XAI_OAUTH_BASE_URL = https://api.x.ai/v1`

## authorize URL 观察

Hermes 构造 authorize URL 时，除了标准 OAuth / PKCE 参数外，还会附带额外参数。

### 标准参数

- `response_type=code`
- `client_id=<XAI_OAUTH_CLIENT_ID>`
- `redirect_uri=<loopback redirect uri>`
- `scope=<XAI_OAUTH_SCOPE>`
- `code_challenge=<pkce challenge>`
- `code_challenge_method=S256`
- `state=<random hex>`
- `nonce=<random hex>`

### Hermes 额外参数

- `plan=generic`
- `referrer=hermes-agent`

## 这些额外参数的意义

从 Hermes 注释和测试可以观察到：

### `plan=generic`

Hermes 明确认为这个参数是必须的，否则 `accounts.x.ai` 会拒绝非 allowlisted
client 的 loopback OAuth。

### `referrer=hermes-agent`

Hermes 用这个值让 xAI 可以在服务端识别这是 Hermes 发起的 OAuth 登录。

## token exchange 请求形状

Hermes 在 token exchange 阶段发送的核心字段包括：

- `grant_type=authorization_code`
- `code=<authorization code>`
- `redirect_uri=<same redirect uri>`
- `client_id=<XAI_OAUTH_CLIENT_ID>`
- `code_verifier=<pkce verifier>`

此外，Hermes 还会额外回传：

- `code_challenge=<pkce challenge>`
- `code_challenge_method=S256`

## 额外回传 challenge 的意义

Hermes 的注释明确提到，xAI 的 OAuth 服务端可能会在 token exchange 阶段再次
校验 `code_challenge`，而不是只依赖 authorize 阶段保存的服务端状态。

也就是说，对于 Hermes 来说，下面这件事是一个重要兼容性特征：

- token exchange 不只发 `code_verifier`
- 还会回传 `code_challenge` 和 `code_challenge_method`

## refresh 请求形状

Hermes 在 refresh 阶段发送的核心字段包括：

- `grant_type=refresh_token`
- `client_id=<XAI_OAUTH_CLIENT_ID>`
- `refresh_token=<saved refresh token>`

## endpoint 校验行为

Hermes 对 discovery 返回的 endpoint 做了非常严格的校验：

- 必须是 `https`
- host 必须是 `x.ai` 或 `*.x.ai`

这意味着 Hermes 把下面这件事看得非常重要：

- 不能盲目信任缓存下来的 `authorization_endpoint` 或 `token_endpoint`

## callback 行为观察

Hermes 当前围绕 loopback callback 采用如下行为：

- 默认使用 `127.0.0.1:56121/callback`
- 在远端环境中支持不自动打开浏览器
- 在浏览器型远端环境中支持 `manual-paste`
- 在 SSH 远端模式下，明确要求把本地端口转发到远端 loopback callback listener

## 状态保存观察

Hermes 会把以下内容保存到认证状态中：

- `tokens`
  - `access_token`
  - `refresh_token`
  - `id_token`
  - `expires_in`
  - `token_type`

- `last_refresh`
- `auth_mode = oauth_pkce`
- `discovery`
- `redirect_uri`

## entitlement / tier 错误观察

Hermes 明确把下面这类错误区分出来：

- `xai_oauth_tier_denied`

它代表的不是“重新登录能解决”的普通认证失败，而是：

- OAuth 虽然完成了
- 但账号没有被允许使用 xAI API / OAuth 能力面

所以在未来 SKILL 设计里，必须区分：

- 需要重新登录
- 已登录但 entitlement 被拒绝

## 对未来 SKILL 的启发

从参数观察角度看，未来 Grok SKILL 至少需要把下面这些点当成重点：

1. 是否采用与 Hermes 一致的 authorize 参数组织方式
2. 是否保留 `plan=generic`
3. 是否保留 `referrer=hermes-agent`
4. token exchange 是否回传 `code_challenge`
5. redirect_uri 是否保持 loopback 一致性
6. refresh 是否继续使用同一 client_id
7. entitlement 403 是否单独分类

## 注意事项

这份文档记录的是“观察到的 Hermes 行为特征”，不是对上游 xAI OAuth 行为的
长期保证。

这些参数和值未来都可能发生变化，因此在真正实现时，需要把它们当成：

- 当前观察到的兼容性输入
- 会变化的外部依赖
- 需要持续验证的行为假设
