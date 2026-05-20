# Rust CLI 设计方案

## 目标

为当前这套 Grok SKILL 能力路由方案设计一个 Rust CLI，作为 SKILL 的脚本执行层。

这个 Rust CLI 的职责不是替代 SKILL，而是为 SKILL 提供一组稳定、可组合、可测试、可重复执行的底层能力。

这里的设计原则已经按 `Hermes` 当前实现收口，重点不是“发明一个新 Grok 接口层”，而是“把 Hermes 已经证明可行的 Grok OAuth 能力路由，变成独立可复用的 SKILL Scripts”。

## 核心定位

可以把它理解为：

- SKILL 负责理解用户意图、判断任务走向、决定什么时候先认证、什么时候恢复任务
- Rust CLI 负责完成确定性的本地执行动作，例如状态检查、OAuth 流程协调、能力调用、结构化结果输出

## 设计原则

### 1. SKILL 负责决策，CLI 负责执行

SKILL 擅长：

- 判断用户是不是要用 Grok
- 判断任务属于聊天、搜索、图片、视频、TTS、STT 还是兼容代理
- 判断当前是否要先做 OAuth
- 决定如何与用户沟通

CLI 擅长：

- 检查本地状态文件
- 按固定协议组织 OAuth 参数
- 读写凭据
- 运行本地 auth runtime，并处理 callback / manual-paste fallback
- 为 `task` / `proxy` 输出共享 runtime credentials 视图
- 输出结构化 JSON
- 执行确定性的子命令

### 2. OAuth-first 必须体现在 CLI 设计上

CLI 默认行为应当是：

- 优先检查和复用 OAuth
- 不静默退回 API key
- entitlement / tier 错误与“需要重新登录”严格区分

说明：

- Hermes 内部很多 xAI 能力支持 OAuth 与 API key 双路径
- 但当前这个 `grok-cli` 的目标是“让用户用自己的 OAuth 来完整使用 Grok”
- 所以 CLI 的主路径必须是 `xai-oauth`

### 3. CLI 必须脚本友好

作为 SKILL scripts，CLI 需要适合自动调用：

- 所有核心结果必须可输出 JSON
- 错误码必须可区分
- 标准输出要面向机器消费
- 标准错误可以保留更详细的人类可读日志

### 4. CLI 必须尽量贴近 Hermes 的分层

不要做成一个只有 `grok use ...` 的巨型命令。

更好的方式是：

- 认证：`auth`
- 能力调用：`task`
- 兼容代理：`proxy`
- 状态与调试：`state` / `debug`

这里专门把 `proxy` 提成一级子命令，是为了和 Hermes 保持一致。因为 Hermes 的 proxy 本质上是独立的本地 HTTP 服务，不属于普通自然语言任务执行面。

## CLI 在整体架构中的位置

建议整体分工如下：

```text
用户
  |
  v
SKILL
  |
  | 负责意图识别、任务分类、前后衔接
  v
Rust CLI
  |
  | 负责状态检查、OAuth 执行、能力调用、JSON 输出
  v
Grok / xAI 能力面
```

未来交互应该类似：

1. 用户说“用 Grok 搜 X 上的讨论”
2. SKILL 判断这是 `x_search`
3. SKILL 调用 Rust CLI 查询 OAuth 状态
4. 如果没有 OAuth，SKILL 调用 Rust CLI 进入认证流程
5. Rust CLI 把结构化状态返回给 SKILL
6. SKILL 继续调用 Rust CLI 执行 `x_search`
7. Rust CLI 返回结果 JSON
8. SKILL 组织最终回答

## CLI 责任边界

Rust CLI 应负责以下任务：

### 1. 认证状态检查

检查：

- 是否存在已保存的 OAuth 状态
- access token 是否存在
- token 是否快过期
- 是否需要 refresh
- 是否处于重新登录状态
- 是否出现 entitlement / tier 拒绝

### 2. OAuth 流程协调

负责：

- 生成 authorize URL
- 管理 PKCE 参数
- 管理本地 HTTP callback runtime
- 支持 manual-paste
- 处理 token exchange
- 处理 refresh
- 保存最终状态

这里应尽量贴近 Hermes 当前的认证分层：

- `auth login` 不应只是临时起一个简化 listener 然后阻塞等待
- callback 接收、manual-paste 解析、exchange、refresh 应该是可复用的 auth runtime 子层
- loopback 回调失败时，应能无缝切换到 manual-paste，而不是让用户回到外部重新拼流程
- token endpoint 的调用不能只做裸 `POST`：需要显式 timeout、请求头、轻量 retry 和失败上下文记录
- 在当前 Grok CLI 落地中，真实验证表明浏览器 / curl 可以成功访问 `auth.x.ai`，但 Rust `reqwest` 默认路径会出现 `connect` 失败；实现上应允许 auth runtime 对共享 HTTP client 做网络路径修正，例如优先绑定 IPv4，以保证 callback 成功后 token exchange / refresh 也能稳定完成

### 2.1 共享 Runtime Credentials

认证完成后，CLI 还需要提供一层共享 runtime credentials 视图，供后续命令消费。

这层职责包括：

- 从状态文件解析出当前 bearer / base_url / provider
- 统一判断 access token 是否可复用
- 按需执行 refresh
- 让 `task` 与 `proxy` 共享同一套运行时凭据，而不是各自重复解析状态文件

### 3. Grok 能力调用

至少应该为这些能力提供一等调用面：

- `chat`
- `x_search`
- `image_gen`
- `video_gen`
- `tts`
- `stt`

### 4. OpenAI-compatible 代理暴露

这里不是“任务能力”，而是另一条产品面：

- 启动本地 HTTP 服务
- 对外暴露 OpenAI-compatible 路径
- 用 OAuth bearer 转发到 xAI
- 保留上游 SSE 流

### 5. 状态持久化

负责读写本地状态文件，包括：

- OAuth tokens
- discovery 信息
- redirect_uri
- 最近刷新时间
- 最后一次认证错误
- auth_mode

### 6. 结构化输出

CLI 每个核心子命令都应支持统一 JSON 输出，供 SKILL 消费。

## 不应由 CLI 负责的事情

以下内容不建议塞进 Rust CLI：

- 自然语言意图识别
- 用户沟通策略
- 任务级追问逻辑
- 长篇解释和上下文管理
- 决定是否切换到非 Grok provider

这些属于 SKILL 层职责。

## Hermes 对齐后的架构结论

通过当前仓库分析，可以把 `grok-cli` 的参考目标拆成两条线：

### 1. 主任务线

即用户把 Grok 当作能力来使用：

- `task chat`
- `task x-search`
- `task image-gen`
- `task video-gen`
- `task tts`
- `task stt`

这条线以 `xai-oauth` 为主，按需直接调用 xAI API。

### 2. 兼容代理线

即用户把 Grok 当作一个 OpenAI-compatible endpoint 给别的客户端使用：

- `proxy start`
- `proxy status`
- `proxy providers`

这条线不是自然语言任务面，而是本地网关面。

### 为什么一定要分开

因为 Hermes 当前就是这样分的：

- `task` / agent runtime 是能力消费层
- `proxy` 是本地转发层

如果强行把两者揉成一个 `task proxy`，后面实现会反而偏离 Hermes。

## 建议的子命令结构

CLI 命令名固定为：

```text
grok-cli
```

建议的一级子命令：

### `auth`

用于认证相关流程。

建议子命令：

- `auth status`
- `auth login`
- `auth refresh`
- `auth logout`
- `auth print-authorize-url`
- `auth exchange-code`

说明：

- `auth login` 是 auth runtime 的主入口，而不只是浏览器打开命令
- `auth exchange-code` 是 auth runtime 的拆分式 / fallback 入口

### `task`

用于统一能力调用入口。

建议子命令：

- `task chat`
- `task x-search`
- `task image-gen`
- `task video-gen`
- `task tts`
- `task stt`

### `proxy`

用于兼容代理生命周期。

建议子命令：

- `proxy start`
- `proxy status`
- `proxy providers`

说明：

- 当前不加 `proxy stop`
- 因为 Hermes 现状没有该命令
- 代理是前台进程，停止方式就是进程结束

### `state`

用于状态读写和调试。

建议子命令：

- `state show`
- `state path`
- `state validate`

### `debug`

用于调试和兼容性研究。

建议子命令：

- `debug authorize-params`
- `debug token-request-shape`
- `debug hermes-observation`

## 命令职责

### `auth status`

输出认证状态，重点用于让 SKILL 判断下一步。

建议返回字段：

- `logged_in`
- `auth_mode`
- `access_token_present`
- `refresh_token_present`
- `access_token_expiring`
- `relogin_required`
- `entitlement_denied`
- `last_refresh`
- `auth_store_path`
- `provider`
- `base_url`

### `auth login`

负责发起完整 OAuth 流程。

建议支持参数：

- `--json`
- `--no-browser`
- `--manual-paste`
- `--timeout <seconds>`

成功后返回：

- `provider`
- `auth_mode`
- `saved`
- `auth_store_path`
- `redirect_uri`
- `base_url`

### `auth refresh`

强制刷新 access token。

适合：

- SKILL 认为当前会话可能过期
- 或在真正执行任务前先做一次预刷新

### `task chat`

这是对 Grok 聊天能力的统一执行入口。

参考 Hermes，应当同时支持：

- 非流式最终结果
- 流式 SSE
- `function_call` / `tool_calls` 归一化

也就是说，`task chat` 不是单纯一个“返回字符串”的接口，而是要允许上层拿到：

- `output_text`
- `finish_reason`
- `tool_calls`
- 流式 `response.*` 事件

### `task x-search`

统一调用 Grok 的 X 搜索能力。

参考 Hermes，应支持：

- `allowed_x_handles`
- `excluded_x_handles`
- `from_date`
- `to_date`
- `enable_image_understanding`
- `enable_video_understanding`

返回结构应贴近 Hermes 当前工具返回：

- `answer`
- `citations`
- `inline_citations`
- `credential_source`

### `task image-gen`

统一调用 Grok 图片生成能力。

参考 Hermes：

- 请求走 `/images/generations`
- 返回值主字段应为 `image`
- `image` 既可能是本地缓存路径，也可能是远程 URL

所以 CLI 设计上不应该把它限制死成“总是写本地文件”。

### `task video-gen`

统一调用 Grok 视频生成能力。

参考 Hermes：

- 请求先 `POST /videos/generations`
- 再轮询 `/videos/{request_id}`
- 成功时主字段为 `video`
- `video` 当前直接是远程 URL

所以 CLI 不应该擅自把视频下载落盘后再改成 `output_path` 风格。

### `task tts`

统一调用 Grok 文本转语音能力。

参考 Hermes：

- 返回结构主字段是 `file_path`
- 默认落盘目录是 `~/.hermes/cache/audio/audio_cache`
- 平台场景下会带 `media_tag`

因此 CLI 应复用：

- `file_path`
- `media_tag`
- `voice_compatible`

### `task stt`

统一调用 Grok 语音转文字能力。

参考 Hermes：

- 成功返回 `success`
- 返回 `transcript`
- 返回 `provider`

因此 CLI 这里不要自造更复杂结构。

### `proxy start`

负责启动前台本地 HTTP 代理。

参考 Hermes 的代理行为：

- 前台运行
- 忽略客户端传入的 `Authorization`
- 自动附加 OAuth bearer
- 响应流不改写
- SSE 原样保留

### `proxy status`

负责输出当前 provider 适配器状态。

### `proxy providers`

负责列出当前内置 provider。

对当前项目而言，至少要覆盖：

- `xai`

## JSON 输出协议

为了让 SKILL 更稳定地消费 Rust CLI，建议统一输出协议。

### 成功输出

```json
{
  "ok": true,
  "command": "auth status",
  "data": {
    "logged_in": true,
    "provider": "xai-oauth"
  }
}
```

### 失败输出

```json
{
  "ok": false,
  "command": "auth refresh",
  "error": {
    "code": "xai_oauth_tier_denied",
    "message": "OAuth account is not authorized for xAI API access",
    "relogin_required": false,
    "entitlement_denied": true
  }
}
```

建议统一保留：

- `ok`
- `command`
- `data`
- `error.code`
- `error.message`
- `error.relogin_required`
- `error.entitlement_denied`

## 状态文件设计

建议不要把状态散落在多个文件里。

第一阶段可以采用单文件 JSON 存储。

因为用户当前是在为一个独立 SKILL 设计 CLI，而不是往 Hermes 仓库本体里打补丁，所以默认路径建议独立于 Hermes：

```text
~/.grok-cli/auth.json
```

如果后续决定做成多 profile，也可扩展为目录结构。

建议保存字段：

- `version`
- `provider`
- `auth_mode`
- `base_url`
- `tokens`
  - `access_token`
  - `refresh_token`
  - `id_token`
  - `expires_in`
  - `token_type`
- `discovery`
  - `authorization_endpoint`
  - `token_endpoint`
- `redirect_uri`
- `last_refresh`
- `last_auth_error`

## OAuth 设计建议

Rust CLI 中的 OAuth 流程建议拆成三段：

### 1. authorize URL 生成

负责：

- 读取 discovery
- 生成 PKCE 参数
- 生成 state 和 nonce
- 组织 authorize URL

### 2. callback 接收

负责：

- 启动 loopback listener
- 或进入 manual-paste 模式
- 接收 code
- 校验 state

### 3. token exchange / refresh

负责：

- authorization_code 交换 token
- refresh_token 刷新 access token
- 分类错误

## 与 Hermes 兼容性观察的关系

这套 Rust CLI 需要参考 Hermes 的参数和值，但方式应当是：

- 把它们视作“兼容性观察输入”
- 把它们封装在实现层
- 不让这些细节污染 SKILL 的自然语言层

也就是说，未来如果需要适配：

- `client_id`
- `scope`
- `redirect_uri` 形状
- authorize 参数
- token exchange 附带字段

这些都应由 Rust CLI 内部控制，而不是由用户在日常使用中直接接触。

## 错误模型设计

建议从一开始就定义清楚错误分类。

至少要区分：

- `auth_missing`
- `auth_expired`
- `auth_refresh_failed`
- `auth_relogin_required`
- `auth_state_mismatch`
- `auth_callback_timeout`
- `xai_oauth_tier_denied`
- `model_capability_mismatch`
- `invalid_request`
- `io_error`
- `path_not_allowed`

这样 SKILL 才能稳定判断下一步是：

- 重新登录
- 改模型
- 提示 entitlement 问题
- 继续当前任务

## 工程结构建议

如果开始实现，建议用下面这样的 Rust 工程结构：

```text
grok-cli/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── commands/
│   │   ├── auth.rs
│   │   ├── task.rs
│   │   ├── proxy.rs
│   │   ├── state.rs
│   │   └── debug.rs
│   ├── oauth/
│   │   ├── mod.rs
│   │   ├── pkce.rs
│   │   ├── authorize.rs
│   │   ├── callback.rs
│   │   ├── exchange.rs
│   │   └── refresh.rs
│   ├── state/
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   ├── load.rs
│   │   └── save.rs
│   ├── xai/
│   │   ├── mod.rs
│   │   ├── chat.rs
│   │   ├── x_search.rs
│   │   ├── image.rs
│   │   ├── video.rs
│   │   ├── tts.rs
│   │   └── stt.rs
│   ├── proxy/
│   │   ├── mod.rs
│   │   ├── server.rs
│   │   ├── handlers.rs
│   │   └── allowed_paths.rs
│   ├── output/
│   │   ├── json.rs
│   │   ├── sse.rs
│   │   └── error.rs
│   └── util/
│       ├── http.rs
│       ├── time.rs
│       └── paths.rs
```

## 第一阶段实现建议

为了避免一开始范围过大，建议分阶段：

### 阶段 1

- `auth status`
- `auth login`
- `auth refresh`
- `state show`
- `state path`
- `state validate`
- 状态文件读写
- 统一 JSON 输出

### 阶段 2

- `task x-search`
- `task image-gen`
- `task tts`
- `task stt`

### 阶段 3

- `task video-gen`
- `task chat`
- `proxy start`
- `proxy status`
- `proxy providers`
- `debug` 子命令

### 为什么这样分阶段

因为这最贴近 Hermes 当前复杂度：

- OAuth 是全部能力的前提
- `x_search` / `image` / `tts` / `stt` 是较容易独立验证的能力面
- `chat --stream` 与 `proxy` 是最容易因为协议边界产生偏差的部分，放到后面更稳

## 为什么 Rust 适合这件事

Rust 适合做这类 SKILL scripts，主要因为：

- 适合写稳定的 CLI
- 错误类型清晰
- 状态管理和 JSON 输出容易约束
- SSE 与本地 HTTP 服务也适合做强约束实现
- 方便后续加测试
- 发布为单二进制文件很方便

## 当前设计结论

按 Hermes 当前行为收口后，这份设计最重要的边界是：

- `grok-cli` 不是新协议发明器，而是 Hermes Grok 能力的独立路由器
- `task` 负责能力调用
- `proxy` 负责兼容代理
- `Responses stream` 的事件形状直接参考 Hermes
- 图片和视频不要强制都转成本地文件
- TTS / STT 的字段名尽量直接复用 Hermes

这能最大程度降低后续“看起来像兼容，实际上行为不同”的风险。

## 下一步建议

这份设计确认后，下一步建议继续产出或继续推进：

1. 按当前已更新的 [rust-cli-command-spec.md](./rust-cli-command-spec.md)
   开始创建 Rust 工程脚手架

2. 以 [rust-cli-state-schema.md](./rust-cli-state-schema.md)
   作为磁盘状态契约

3. 补一份 `phase-1` 实施计划，把 OAuth 与状态层先做出来
