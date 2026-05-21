# 故障排查

## 1. `state_file_missing`

含义：
- 当前没有可用状态文件

处理：
- 先执行 `login --json`

## 2. `auth_relogin_required`

含义：
- 当前 refresh 已无法恢复，必须重新登录

处理：
- 执行 `login --json`

## 3. `xai_oauth_tier_denied`

含义：
- 当前 OAuth 账号没有对应 API / 能力权限

处理：
- 不要重复刷新
- 不要只做重登重试
- 先确认账号订阅、权限和能力开通状态

补充说明：

- 如果错误正文包含 `The OAuth2 access token could not be validated`，它未必真的是订阅层级不足
- `2026-05-20` 的真实媒体验证中，这类错误曾由“access token 已接近过期”触发，`refresh` 后恢复正常
- 当前媒体请求入口已增加“即将过期先 refresh”的自动编排，用来降低这类误判
- 如果你是在旧二进制上复现到这个错误，先升级到包含该自动编排的版本，再重新跑媒体命令

## 4. 浏览器登录页成功，但 CLI 后续失败

常见现象：
- 浏览器已显示连接成功
- 但 CLI 在 token exchange 或 refresh 阶段失败

当前项目中的已知真实根因：
- 浏览器与 `curl` 可以成功访问 `auth.x.ai`
- Rust `reqwest` 在当前环境下可能先走异常的 IPv6 路径

当前实现中的处理：
- 共享 HTTP client 已改为优先绑定 IPv4 出站

## 5. `stt` 报文件不存在

含义：
- `--file` 指向的本地音频文件不存在

处理：
- 确认路径是绝对路径或当前目录下真实存在的文件

## 6. `chat` 或 `search` 的流式输出返回异常

处理顺序建议：

1. 先用非流式 `chat --json` 验证主路径
2. 如果 `chat` 有问题，再测 `chat --stream`、`chat --no-stream` 或 `chat --raw-stream`
3. 如果 `search` 有问题，再测 `search --json`、`search --no-stream` 或 `search --raw-stream`

这样更容易区分：
- 是认证问题
- 是 Responses API 问题
- 还是 SSE 事件本身的兼容问题

## 7. `stt` 返回 `Field 'language' is required when 'format' is true`

含义：
- 当前 STT 请求启用了格式化输出，xAI 要求同时提供 `language`

当前实现中的处理：
- `stt` 已在 multipart 请求里默认补 `language=en`

如果还要显式指定：
- 直接传 `--language <code>`
