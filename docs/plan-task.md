# Grok CLI Plan Task

## 目标

把 `grok-cli` 从文档设计推进到可运行实现，并最终作为 Grok SKILL 的确定性执行层。

这份任务清单基于当前目录中的以下文档重新规划：

- `technical-architecture.md`
- `rust-cli-command-spec.md`
- `rust-cli-state-schema.md`
- `rust-cli-design.md`
- `grok-skill-routing-plan.md`
- `capability-matrix.md`
- `constraints-and-risks.md`

这些早期设计文档现已归档到 [`docs/archive/`](./archive/index.md)，当前公开使用入口以根目录 README 和 `docs/` 下的用户文档为准。

## 当前执行原则

- 不再走“先最小可运行、后续再重构”的路线
- 从第一版开始就按目标项目结构建仓
- 第一版就确定核心依赖、统一错误模型、统一 JSON 输出协议
- 先打通 OAuth / 状态 / 请求执行基础设施，再往上接能力面
- OAuth 层按 Hermes 式 auth runtime + runtime credential resolver 收口，而不是停留在简化 listener
- 明确区分 `auth_relogin_required` 与 `xai_oauth_tier_denied`

## 当前阶段

- 阶段：Phase 16
- 状态：已完成 Phase 16.1-16.6，SuperGrok 媒体主能力补齐进入收尾验收
- 本轮重点：继续补 streaming STT 深水区回归、真实视频编辑/扩展验收和开源发布配套

## 总体任务地图

### Phase 0. 文档冻结与实施入口

目标：把会影响代码骨架的设计决策先冻结，避免实现期来回返工。

- [x] 明确 Grok / Hermes 对齐范围
- [x] 明确 OAuth-first 设计原则
- [x] 明确第一轮开发期命令结构：`auth` / `state` / `usage` / `task`
- [x] Phase 10 已将公开发布命令面扁平化为一级命令：`login` / `status` / `chat` / `search` / `image` / `video` / `tts` / `stt` / `usage` / `model` / `state`
- [x] 明确状态文件 schema v1
- [x] 明确目标项目结构与核心依赖策略
- [x] 冻结第一版 `Cargo.toml` 依赖和 feature 集
- [x] 冻结统一 JSON 成功 / 失败信封
- [x] 冻结退出码与错误码映射表

Phase 0 完成标准：

- 实现时不再讨论“要不要先单文件”
- `Cargo.toml`、输出模型、错误模型可以直接抄进项目

### Phase 1. 工程骨架与共享基础设施

目标：建立可长期演进的 Rust 工程骨架，而不是临时脚手架。

- [x] 初始化 `grok-cli/` Rust 工程
- [x] 落地目标目录结构：
  - [x] `src/main.rs`
  - [x] `src/app.rs`
  - [x] `src/cli.rs`
  - [x] `src/args.rs`
  - [x] `src/error.rs`
  - [x] `src/output.rs`
  - [x] `src/state/`
  - [x] `src/auth/`
  - [x] `src/task/`
- [x] 按架构文档写入首版 `Cargo.toml`
- [x] 初始化 `AppContext`
- [x] 初始化共享 `reqwest::Client`
- [x] 初始化 `tracing` / `tracing-subscriber`
- [x] 接通顶层命令解析与分发
- [x] 增加基础冒烟测试框架

Phase 1 完成标准：

- `cargo check` 通过
- `grok-cli --help` 可用，并在 Phase 10 后直接展示一级命令
- 命令入口、日志、共享上下文已经稳定，不需要后续推倒重来

### Phase 2. 统一输出、错误模型、状态文件基础层

目标：先把 CLI 的“机器可消费外壳”做稳。

- [x] 实现统一 JSON 成功信封
- [x] 实现统一 JSON 失败信封
- [x] 实现退出码映射
- [x] 定义顶层错误类型与领域错误枚举
- [x] 实现敏感字段脱敏输出规则
- [x] 实现状态文件路径解析
- [x] 实现状态文件读取
- [x] 实现状态文件写入
- [x] 实现原子写入策略
- [x] 实现 schema 校验
- [x] 实现运行时派生状态判断：
  - [x] `logged_in`
  - [x] `access_token_present`
  - [x] `refresh_token_present`
  - [x] `access_token_expiring`
  - [x] `relogin_required`
  - [x] `entitlement_denied`
- [x] 实现 `state` 脱敏状态摘要
- [x] 删除公开 `state path` / `state validate` 子入口
- [x] 实现 `auth status`

Phase 2 完成标准：

- `grok-cli status --json` 能输出结构化状态
- `grok-cli state --json` 能输出脱敏状态摘要
- `grok-cli state --help` 不再展示 `path` / `validate` 子命令

### Phase 3. OAuth 基础流程

目标：把登录、刷新、重登判断做成真正可复用的认证层。

阶段说明：

- 这一阶段的目标不再只是“命令能登录”
- 要对齐 Hermes 的思路，形成独立的 auth runtime 和后续可共享的 runtime credential resolver
- 后续所有真实能力都应建立在这层之上，而不是各自重做 token 解析与刷新

- [x] 实现 PKCE verifier / challenge 生成
- [x] 实现 OAuth `state` 参数生成与校验
- [x] 实现 authorize URL 构造
- [x] 实现本地 callback listener
- [x] 升级为本地 HTTP callback runtime
- [x] 实现浏览器启动流程
- [x] 实现 `--no-browser`
- [x] 实现 `--manual-paste`
- [x] 实现 loopback -> manual-paste fallback 编排
- [x] 实现 authorization code exchange
- [x] 实现 token 持久化回写
- [x] 实现 refresh token 流程
- [x] 实现共享 runtime credential resolver
- [x] 实现 `last_refresh` 写回
- [x] 实现 `last_auth_error` 写回
- [x] 实现 `auth print-authorize-url`
- [x] 实现 `auth exchange-code`
- [x] 实现 `auth login`
- [x] 实现 `auth refresh`
- [x] 实现 `auth logout`
- [x] 明确并实现以下错误区分：
  - [x] `auth_refresh_failed`
  - [x] `auth_relogin_required`
  - [x] `auth_state_mismatch`
  - [x] `auth_callback_timeout`
  - [x] `auth_token_exchange_failed`
  - [x] `xai_oauth_tier_denied`

Phase 3 完成标准：

- 首次登录可以落盘得到有效状态文件
- refresh 成功时可以自动更新状态
- refresh 失败时能明确区分“需要重登”与“权限拒绝”
- 浏览器未命中 loopback 时，可用手工复制的授权码继续完成 token exchange
- 后续 `task` 与其他真实能力入口都可以直接复用同一份 runtime credentials，而不是再次手写认证判断

本轮实测补充：

- `2026-05-19` 已通过真实浏览器完成授权，并在 loopback 未命中的情况下，使用页面给出的真实授权码成功执行 `auth exchange-code`
- 同一份真实状态文件已验证 `auth status` 成功，且 `auth refresh` 在后续重试中成功
- `2026-05-19` 已补齐 callback runtime 的浏览器预检 / CORS 兼容，并完成自动 loopback 回调实测：真实浏览器不再停在“复制 code”作为主路径，而是能直接命中本地 callback；当前剩余问题收敛为后续 `https://auth.x.ai/oauth2/token` 请求仍偶发失败，需要在 Phase 4 前补充请求级观测与重试策略
- `2026-05-19` 已为 `auth exchange-code` / `auth refresh` 补上显式 timeout、JSON accept header、轻量 transport retry，以及失败时写回 endpoint / phase / grant_type / redirect_uri 等上下文，便于继续定位真实 token endpoint 的偶发失败
- `2026-05-19` 已完成根因定位：浏览器与 `curl` 访问 `auth.x.ai` 正常，但 Rust `reqwest` 默认路径在当前环境下会对同一 endpoint 报 `connect` 失败；共享 HTTP client 切到 IPv4 出站后，真实 `auth refresh` 与真实浏览器完整登录均已再次验证成功

### Phase 4. 上游客户端与首个能力面

目标：先建立共享 xAI 请求执行层，再用它打通第一个真实能力。

- [x] 实现共享 xAI upstream client
- [x] 实现 bearer 注入
- [x] 实现基础请求构造器
- [x] 实现统一响应解析入口
- [x] 实现统一请求失败映射
- [x] 实现 `task` 公共执行框架
- [x] 实现 `task x-search`
- [x] 对齐 `x_search` 返回结构：
  - [x] `answer`
  - [x] `citations`
  - [x] `inline_citations`
  - [x] `credential_source`
- [x] 增加 `x-search` 冒烟与错误路径测试

Phase 4 完成标准：

- [x] 已登录状态下可真实执行 `task x-search`
- 请求失败、认证失效、entitlement 拒绝都能映射到统一错误模型

本轮补充：

- `2026-05-19` 已补上状态文件原子写入，避免 OAuth callback / refresh 场景读到半成品状态
- `2026-05-19` 已为 `task x-search` 增加本地 stub 冒烟与错误路径测试，覆盖成功返回、403 entitlement 拒绝、缺失 message answer 三类核心路径

### Phase 5. 媒体能力面

目标：把 Hermes 已验证可行的媒体能力纳入同一执行框架。

- [x] 实现 `task image-gen`
- [x] 实现图片结果结构统一：
  - [x] 本地路径返回
  - [x] 远程 URL 返回
- [x] 实现 `task video-gen`
- [x] 实现视频生成轮询模型
- [x] 统一视频结果结构
- [x] 实现 `task tts`
- [x] 对齐音频文件输出结构
- [x] 实现 `task stt`
- [x] 对齐 transcript 输出结构
- [x] 为 image / video / tts / stt 增加统一错误映射

Phase 5 完成标准：

- 四类媒体能力都能通过同一 OAuth 状态和同一 upstream client 执行
- 返回结构可被 SKILL 稳定消费

本轮补充：

- `2026-05-19` 已实现 `task image-gen`：复用共享 runtime credentials 和统一 JSON upstream 执行层，请求 `POST /images/generations`
- `2026-05-19` 图片结果现已同时支持两种稳定结构：
  - 默认返回远程 `image` URL
  - 指定 `--output-file` 时请求 `b64_json` 并落盘，返回本地 `image` 路径
- `2026-05-19` 已补上 `task image-gen` 参数校验、模块级解析测试与命令级 stub 冒烟测试
- `2026-05-19` 已实现 `task video-gen`：先请求 `POST /videos/generations`，再轮询 `GET /videos/{request_id}`，成功时返回远程 `video` URL
- `2026-05-19` 视频结果结构已按文档收口为 `video` / `modality` / `aspect_ratio` / `duration` / `extra.request_id` / `extra.resolution`
- `2026-05-19` 已补上 `task video-gen` 参数校验、模块级解析测试与命令级 stub 测试，覆盖成功轮询和失败状态返回
- `2026-05-19` 已实现 `task tts`：调用 `POST /tts` 接收原始音频字节，按 Hermes 约定写入本地文件并返回 `file_path` / `media_tag` / `voice_compatible`
- `2026-05-19` 已实现 `task stt`：调用 `POST /stt` multipart 上传音频文件，成功返回 `transcript`
- `2026-05-19` 媒体能力现已统一复用共享 OAuth 状态、共享 upstream client 和统一错误信封；相关命令级测试已覆盖 image / video / tts / stt 的成功与关键错误路径

### Phase 6. 聊天与流式输出

目标：补齐主聊天路径，并把直接 `task` 调用收口为唯一主消费路径。

- [x] 实现 `task chat` 非流式
- [x] 实现 `task chat --stream`
- [x] 对齐 `codex_responses` 主运行时
- [x] 为 `task chat` 默认注入通用 `web_search`
- [x] 保留 `task x-search` 作为 X 专用入口
- [x] 为 `task chat` 增加 `--with-x-search` 混合模式
- [x] 实现 `response.*` 事件流输出
- [x] 实现 tool-calling 归一化

Phase 6 完成标准：

- `task chat` 可流式、可非流式运行
- `task` 没有额外兼容层依赖，SKILL 可直接稳定消费 JSON 输出

本轮补充：

- `2026-05-19` 已实现 `task chat` 非流式：走 `POST /responses`，返回 `protocol` / `output_text` / `finish_reason` / `tool_calls`
- `2026-05-19` 已实现 `task chat --stream`：走同一 `codex_responses` 主路径，并将上游 SSE 事件整理输出为 `response.output_text.delta` / `response.output_text.done` / `response.output_item.done` / `response.completed` / `response.failed`
- `2026-05-19` 已完成 `function_call` -> OpenAI 风格 `tool_calls` 归一化，非流式和流式路径都可稳定消费
- `2026-05-19` 已补上 `task chat` 模块级解析测试与命令级 stub 测试，覆盖非流式文本、非流式 tool-call、流式 SSE 三条主路径
- `2026-05-20` 已按 Hermes 分层补齐 chat 搜索模式：`task chat` 默认注入通用 `web_search`，`task x-search` 继续作为 X 专用入口保留，另新增 `task chat --with-x-search` 混合模式，用于同时挂载 `web_search + x_search`
- `2026-05-20` 已补 `task chat` 回归测试，覆盖默认 `web_search`、`--no-web-search` 纯聊天、`--with-x-search` 混合模式三条路径，并同步更新命令参考、快速开始、样例与验收文档
- `2026-05-20` 已确认 SKILL 直接消费 CLI JSON 输出后，`proxy` 兼容层不再提供必要价值；现已将 `proxy` 命令、实现、测试与文档整体下线。Phase 10 后主路径进一步扁平化为 `login` / `status` / `chat` / `search` / `image` / `video` / `tts` / `stt` / `usage`

### Phase 7. SKILL 集成、测试与交付

目标：把 CLI 从“能运行”推进到“可接入、可维护、可验收”。

- [x] 明确 SKILL 到 CLI 的命令调用约定
- [x] 明确认证后恢复原始任务的调用方式
- [x] 为核心命令补集成测试
- [x] 为错误场景补回归测试
- [x] 补示例状态文件与样例输出
- [x] 补中文使用文档
- [x] 补故障排查说明
- [x] 补验收样例
- [x] 梳理后续能力扩展接口

Phase 7 完成标准：

- SKILL 可以稳定依赖 `grok-cli` 的 JSON 输出与错误码
- 项目具备最基本的维护、排障和验收材料

本轮补充：

- `2026-05-20` 已在 `grok-cli/docs/` 下补齐中文交付文档，并使用 `docs/index.md` 作为总索引，覆盖快速开始、命令参考、SKILL 集成约定、样例、故障排查、验收样例；在确认 SKILL 不再需要 OpenAI-compatible 本地桥接后，已同步移除 `proxy` 相关章节
- `2026-05-20` 已补示例状态文件与样例输出：
  - `.sample/auth.json`
  - `.sample/auth-with-pending-oauth.json`
- `2026-05-20` 已补 Phase 7 契约 / 回归测试：
  - `tests/contract_regressions.rs`
  - 覆盖顶层帮助、任务帮助、错误信封等稳定接口
- `2026-05-20` 已补 `docs/extension-points.md`，收口当前稳定契约、推荐扩展方向、新能力接入流程和不建议轻易改动的接口面
- `2026-05-20` 已确认 `debug` 命令主要是为早期 spec 未冻结时补充构造验证而存在；随着 spec、文档与回归测试补齐，现已将 `debug` 命令、实现与文档整体下线

### Phase 8. Usage / Quota / Session Accounting

目标：新增一个真正可用的 `usage` 命令，同时补齐本地 session 账本、成本估算，并明确不再实现猜测性的账号额度展示。

- [x] 冻结 `usage` 命令规格与 JSON 输出结构
- [x] 确定 `usage` 为顶层命令，而不是挂到 `auth` / `task`
- [x] 设计并落地 `src/usage/` 模块结构
- [x] 设计并落地 SQLite `session.db` 路径与 schema
- [x] 实现 active session 解析规则
- [x] 为 `task` 能力调用补本地 usage instrumentation
- [x] 捕获并持久化最近一次 rate-limit headers
- [x] 建立本地 pricing table 与成本估算逻辑
- [x] 删除 xAI provider-specific live quota / Account limits 实现
- [x] 明确 `usage` 只展示本地 SQLite session 统计与最近 rate-limit 快照
- [x] 实现 `usage --json`
- [x] 实现 `usage` 人类可读输出
- [x] 为空 session / 本地统计 / schema 错误补回归测试
- [x] 补文档样例与验收说明
- [x] 对齐 Hermes 当前实现边界：不再假设存在可复用的 xAI account limits API
- [x] 基于真实 xAI 404 实测和 Hermes 源码核对，收口为 disabled 方案

Phase 8 完成标准：

- `grok-cli usage --json` 可以稳定返回本地 session 累计 usage
- 输出不查询、不展示、不返回 Account limits
- 命令只依赖本地统计，不因远程额度接口不可用而失败
- `task` 的真实上游请求会沉淀到同一份 session 账本

本轮规划补充：

- `2026-05-20` 已新增 `docs/usage-command-spec.md`，冻结 `usage` 顶层命令、统一 JSON 结构、SQLite schema 草案、rate-limit snapshot 与本地统计策略
- `2026-05-20` 已更新 `grok-cli/docs/index.md` 与 `grok-cli/docs/command-reference.md`，把 `usage` 纳入正式文档索引与命令面
- `2026-05-20` 已完成 Phase 8 第一轮实现：新增顶层 `usage` 命令、`src/usage/` 本地 session accounting、SQLite `session.db`、`task` usage instrumentation、recent rate-limit snapshot 持久化
- `2026-05-20` 已把 `usage` 的人类输出收口为分组树形摘要，风格对齐目标样例；本地成本估算和默认上下文窗口按当前 Grok 模型映射输出；`docs/samples.md` 与 `docs/acceptance.md` 已同步新增 `usage` 样例与验收项
- `2026-05-20` 已核对 Hermes 当前源码：`/usage` 的 account limits 正式实现不包含 `xai` / `xai-oauth` 分支；结合真实 xAI OAuth `usage` 探测对 `https://api.x.ai/v1/entitlements` 与 `https://api.x.ai/v1/usage` 均返回 `404 Not Found`，当前方案已正式收口为删除 xAI live quota / Account limits 输出

### Phase 9. Model Switching Alignment

目标：只为文本任务开放默认模型切换，避免把 Hermes 的媒体 provider 选择体系误做成统一的全局 model 切换。

- [x] 新增顶层 `model` 命令
- [x] 将 `model show` / `model list` / `model set` 收口为单一 `grok-cli model` 入口
- [x] `grok-cli model` 默认展示模型列表和当前已选共享文本模型
- [x] `grok-cli model` 支持方向键上下选择模型，并提供 `exit`
- [x] `grok-cli mode` 作为 `grok-cli model` 的别名
- [x] `grok-cli model --model <MODEL>` 支持脚本化选择共享文本模型
- [x] 为 `task chat` 接入默认模型覆盖
- [x] 为 `task x-search` 接入默认模型覆盖
- [x] 明确 `chat` / `search` 使用同一个默认模型
- [x] 明确媒体能力继续使用命令级显式 `--model`
- [x] 补文档索引、命令参考、快速开始、样例、验收
- [x] 补回归测试，覆盖成功设置与非法任务拒绝
- [x] 对照 Hermes 源码核对媒体能力的参数、返回与执行逻辑边界

Phase 9 完成标准：

- `model` 不再暴露 `show` / `list` / `set` 子命令
- `model` 不再错误地暴露 `image-gen` / `video-gen` / `tts` / `stt`
- 文档明确区分“共享文本默认模型切换”和“媒体命令显式模型参数”
- 对 Hermes 的对照结论明确记录哪些能力是一致的，哪些仍有实现差异

本轮补充：

- `2026-05-20` 已把 `model` 命令收口为共享文本模型选择；`chat` 与 `search` 总是使用同一个默认模型，媒体命令虽然仍保留各自的 `--model` 参数，但不会再读取共享文本模型表
- `2026-05-20` 已补 `tests/model_commands.rs` 回归，确保 `model --model` 同时写入 `text` / `chat` / `search`，并确认 `mode` 别名可用
- `2026-05-20` 已同步更新 `docs/index.md`、`docs/command-reference.md`、`docs/quickstart.md`、`docs/samples.md`、`docs/acceptance.md`
- `2026-05-20` 已完成与 Hermes 源码的媒体能力对照：Hermes 的图片 / 视频是 provider 插件式 surface，TTS / STT 是工具路由 surface，不存在统一的“所有媒体默认模型都由 `model` 管理”的实现，因此当前 CLI 收口为“文本命令可切默认模型，媒体命令继续显式传参”更接近 Hermes 的真实边界
- `2026-05-20` 已按 Hermes/xAI 请求形状收口媒体主路径：
  - `task tts` 改为发送 `text` / `voice_id` / `language`
  - `task stt` 改为 multipart `file + format + language`，不再发送 `model`
  - `task video-gen` 改为 `image: {url}` / `reference_images: [{url}]`，并补 `image-url` 与 `reference-image-url` 互斥约束
- `2026-05-20` 已完成真实媒体验证：
  - `task image-gen` 成功返回真实 xAI CDN 图片
  - `task tts` 成功生成真实 MP3
  - `task stt` 成功转写上一条真实 MP3
  - `task video-gen` 已真实跑通 text-to-video、image-to-video、reference-image video 三条分支
- `2026-05-20` 已定位真实媒体链路的剩余认证风险：视频接口在 access token 临近过期时更容易直接返回 token validation failure；现已在共享 upstream auth options 中为媒体命令启用“即将过期先 refresh”的自动编排
- `2026-05-20` 已把“即将过期先 refresh”正式接入媒体命令主路径：`task image-gen`、`task video-gen`、`task tts`、`task stt` 都通过共享 runtime credential resolver 在发请求前检查 token 是否临近过期，必要时先执行 refresh 再继续真实请求
- `2026-05-20` 已完成这轮真实回归结论沉淀：先前视频分支报出的 `The OAuth2 access token could not be validated. [WKE=unauthenticated:bad-credentials]`，根因不是媒体参数形状错误，而是 access token 临近过期时视频接口更敏感；刷新后 text-to-video、image-to-video、reference-image video 都恢复成功
- `2026-05-20` 已完成发布版入口调整后的真实媒体验证：
  - `grok-cli image` 使用老人厨房中文提示词成功生成本地图片 `.tmp/media-verification/elder-kitchen.png`
  - `grok-cli image` 使用同提示词成功返回远程 xAI 图片 URL，用于 image-to-video
  - `grok-cli video` 使用赛博朋克雨夜街道中文提示词成功生成 text-to-video MP4
  - `grok-cli video --image-url ...` 使用老人厨房图片成功生成 image-to-video MP4
  - `grok-cli tts` 成功生成 `.tmp/media-verification/tts-sample.mp3`
  - `grok-cli stt` 成功转写上述 MP3

### Phase 10. CLI Product Surface Simplification

目标：把 CLI 从开发期的 `auth` / `task` 二级命令结构，收口为面向用户和发布的扁平一级命令。

- [x] 将 `auth login` 扁平化为 `login`
- [x] 将 `auth status` 扁平化为 `status`
- [x] 将 `auth refresh` 扁平化为 `refresh`
- [x] 将 `auth logout` 扁平化为 `logout`
- [x] 将 `auth print-authorize-url` 扁平化为 `print-authorize-url`
- [x] 将 `auth exchange-code` 扁平化为 `exchange-code`
- [x] 将 `task chat` 扁平化为 `chat`
- [x] 将 `task x-search` 扁平化为 `search`
- [x] 将 `task image-gen` 扁平化为 `image`
- [x] 将 `task video-gen` 扁平化为 `video`
- [x] 将 `task tts` 扁平化为 `tts`
- [x] 将 `task stt` 扁平化为 `stt`
- [x] 将 `model` 的公开参数收口为 `--model <MODEL>`
- [x] 保留 `--task` 作为 `--command` 的兼容别名，避免旧脚本立即断裂
- [x] 更新 usage 账本的新命令名，并兼容旧 SQLite 里的 `task ...` 历史记录分类
- [x] 更新 README、docs、样例、命令参考、验收文档
- [x] 更新契约测试和命令级回归测试
- [x] 重新安装本地 release CLI 到 `/Users/seanmo/.cargo/bin/grok-cli`

Phase 10 完成标准：

- `grok-cli --help` 直接展示一级命令，不再展示 `auth` 或 `task`
- `grok-cli chat --help`、`grok-cli search --help`、`grok-cli usage --help` 可用
- `grok-cli model --help` 描述共享文本模型默认值，且不展示 `show` / `list` / `set` 子命令
- 旧的 `task ...` 用户入口不再存在
- 完整 `cargo test --quiet` 全绿

本轮补充：

- `2026-05-20` 已选择并执行“方案 2：产品化扁平 CLI，删除公开 `task` 层”
- `2026-05-20` 已确认本机安装后的 `which grok-cli` 为 `/Users/seanmo/.cargo/bin/grok-cli`
- `2026-05-20` 已确认安装版 `grok-cli --help` 输出一级命令：`login`、`status`、`refresh`、`logout`、`print-authorize-url`、`exchange-code`、`state`、`model`、`usage`、`chat`、`search`、`image`、`video`、`tts`、`stt`
- `2026-05-20` 已完成 `cargo test --quiet` 与 `cargo build --release --quiet`

### Phase 11. Documentation Packaging

目标：把仓库整理成可发布、可阅读、不会被历史设计文档干扰的 GitHub 项目结构。

- [x] 重写根目录英文 `README.md`
- [x] 新增根目录中文 `README.zh-CN.md`
- [x] 在英文 README 顶部加入中文 README 链接
- [x] 在中文 README 顶部加入英文 README 链接
- [x] 将根目录早期设计文档统一迁移到 `docs/archive/`
- [x] 新增 `docs/archive/index.md` 作为历史文档索引
- [x] 更新 `docs/index.md` 作为当前文档总入口
- [x] 将任务清单放入 `docs/plan-task.md` 并继续维护勾选状态
- [x] 更新 README、快速开始、命令参考、发布指南中的用户示例为扁平命令
- [x] 标注归档文档可能包含旧的 `auth ...` / `task ...` / `proxy` / `debug` 入口，避免误导当前用户

Phase 11 完成标准：

- 根目录只保留项目入口文件、Cargo 文件、源码、测试和 `docs/`
- 当前用户优先阅读 `README.md`、`README.zh-CN.md`、`docs/index.md`
- 历史设计文档只从 `docs/archive/index.md` 进入
- 当前公开命令示例优先使用 `grok-cli login`、`grok-cli chat "..."`、`grok-cli search "..."` 等一级命令

本轮补充：

- `2026-05-20` 已确认 `/Users/seanmo/AI/develop/Grok` 根目录只保留 `grok-cli/` 项目目录
- `2026-05-20` 已确认 `grok-cli/` 根目录只保留 README、Cargo、源码、测试、样例、临时目录和 `docs/`，不再散落早期 Markdown 设计文档
- `2026-05-20` 已确认早期设计文档全部收口到 `docs/archive/`

### Phase 12. Docs Information Architecture

目标：把 `docs/` 从“文档集合”升级为清晰可导航的信息架构，并让每个公开 CLI 命令都有自己的 spec 和使用页。

- [x] 新建 `docs/commands/` 存放每个 CLI 命令的 spec / 使用文档
- [x] 新建 `docs/guides/` 存放快速开始、故障排查、发布安装指南
- [x] 新建 `docs/reference/` 存放样例输出、usage 深度规格、SKILL 集成、扩展边界
- [x] 新建 `docs/project/` 存放验收样例等项目管理资料
- [x] 保留 `docs/archive/` 作为历史设计归档
- [x] 保留 `docs/plan-task.md` 在 `docs/` 根部，方便持续追踪和打勾
- [x] 为 `login` 新增命令 spec
- [x] 为 `status` 新增命令 spec
- [x] 为 `refresh` 新增命令 spec
- [x] 为 `logout` 新增命令 spec
- [x] 为 `print-authorize-url` 新增命令 spec
- [x] 为 `exchange-code` 新增命令 spec
- [x] 为 `state` 新增命令 spec，收口为直接展示脱敏状态摘要
- [x] 为 `model` 新增命令 spec，覆盖单入口列表展示、交互选择与脚本化选择
- [x] 为 `usage` 新增命令 spec
- [x] 为 `chat` 新增命令 spec
- [x] 为 `search` 新增命令 spec
- [x] 为 `image` 新增命令 spec
- [x] 为 `video` 新增命令 spec
- [x] 为 `tts` 新增命令 spec
- [x] 为 `stt` 新增命令 spec
- [x] 重写 `docs/index.md`，增加每个 CLI 命令到对应 markdown 的入口
- [x] 重写 `docs/commands/index.md`，作为命令 spec 总览
- [x] 修正 README、中文 README、快速开始、归档索引、样例文档中的新路径链接
- [x] 完成 Markdown 相对链接检查

Phase 12 完成标准：

- 打开 `docs/index.md` 能看到完整文档结构
- 每个公开顶层命令都能从 `docs/index.md` 跳到对应 spec
- `commands/index.md` 只做命令索引，不再承载所有详细参数
- 用户指南、参考规格、项目资料和历史归档边界清楚
- Markdown 相对链接检查无断链

本轮合并与简化结论：

- `commands/index.md` 与单命令文档拆开：索引负责导航，单命令文档负责 spec 和使用。
- `quickstart` 不再重复完整参数，只保留最快上手路径。
- `samples` 保留输出样例，不再承担命令参考职责。
- `usage-command-spec` 保留为深度设计；普通用户看 `commands/usage.md`。
- `acceptance` 放入 `project/`，不进入用户主路径。
- `archive` 继续保留历史设计，不和当前公开命令文档混在一起。

### Phase 13. Auth Debug Surface Cleanup

目标：进一步简化用户看到的认证命令面，让普通用户只需要理解 `login` / `status` / `refresh` / `logout`，把早期调试入口从公开产品面移走。

- [x] 从 CLI 顶层公开命令中移除 `print-authorize-url`
- [x] 保留 `login --manual-paste` 输出 authorize URL 和 pending OAuth session，覆盖原 `print-authorize-url` 的排障价值
- [x] 将 `exchange-code` 标记为隐藏命令，不再出现在 `grok-cli --help`
- [x] 保留 `exchange-code` 的执行能力，用于 loopback / manual-paste 异常救援
- [x] 删除 `docs/commands/print-authorize-url.md`
- [x] 删除公开命令页 `docs/commands/exchange-code.md`
- [x] 新增 `docs/reference/internal-auth.md`，记录隐藏认证救援入口
- [x] 更新 README / 中文 README，不再列出 `print-authorize-url` 和 `exchange-code`
- [x] 更新 `docs/index.md` 和 `docs/commands/index.md`，公开命令列表不再包含这两个入口
- [x] 更新 `docs/reference/extension-points.md` 和 `docs/reference/usage-command-spec.md`
- [x] 更新认证测试：使用 `login --manual-paste` 验证 authorize URL 形状
- [x] 更新 help 回归测试：确认 `print-authorize-url` / `exchange-code` 不出现在顶层 help

Phase 13 完成标准：

- `grok-cli --help` 不显示 `print-authorize-url`
- `grok-cli --help` 不显示 `exchange-code`
- `grok-cli print-authorize-url` 不再是可用命令
- `grok-cli exchange-code` 仍可作为隐藏救援命令运行
- 公开 README 和命令索引只展示日常用户需要的认证命令
- 内部救援路径有单独 reference 文档说明

### Phase 14. Text Streaming Defaults

目标：把文本命令收口为“人类默认流式、自动化默认 JSON 非流式”的一致产品行为，同时避免打破 SKILL 的稳定消费契约。

- [x] 为 `chat` 增加统一默认流式判定
- [x] 为 `search` 增加统一默认流式判定
- [x] 为 `search` 接入 Responses SSE 流式执行路径
- [x] 保留 `--stream` 显式强制流式能力
- [x] 新增 `--no-stream` 显式关闭默认流式能力
- [x] 新增 `--raw-stream` 作为原始 SSE 事件输出入口
- [x] 抽取共享文本流式执行器，避免 `chat` / `search` 重复实现
- [x] 保持 `--json` 默认返回稳定单次结构化结果
- [x] 将非流式人类输出改为正文优先的排版结果
- [x] 补命令级回归测试，覆盖 `chat` / `search` 的流式与非流式主路径
- [x] 补文档、README、快速开始、排障和 SKILL 契约说明

Phase 14 完成标准：

- `grok-cli chat "..."` 默认流式输出
- `grok-cli search "..."` 默认流式输出
- 默认人类流式输出应直接显示正文，而不是原始 `event/data` 包装
- `grok-cli chat --stream ...` 与 `grok-cli search --stream ...` 显式使用格式化流式输出
- `grok-cli chat --raw-stream ...` 与 `grok-cli search --raw-stream ...` 输出原始 SSE 事件
- `grok-cli chat --json ...` 与 `grok-cli search --json ...` 默认返回稳定单次结果
- `grok-cli chat --no-stream ...` 与 `grok-cli search --no-stream ...` 可显式关闭流式
- SKILL 文档明确要求自动化固定使用 `--json`

本轮补充：

- `2026-05-20` 已将文本命令流式判定统一收口为共享逻辑：非 `--json` 默认流式，`--json` 默认非流式，`--stream` / `--no-stream` 可显式覆盖
- `2026-05-20` 已为 `search` 接入与 `chat` 相同的 Responses SSE 主路径，并抽取 `src/task/text_stream.rs` 作为共享文本流式执行器
- `2026-05-20` 已将默认人类流式渲染收口为“正文直出”，不再把原始 `event:` / `data:` 包装直接打印到屏幕；显式 `--raw-stream` 保留原始事件流入口
- `2026-05-20` 已把 `--stream` 调整为格式化流式输出的显式声明，并将 `--no-stream` 的人类输出改为正文优先、带简短模型 / 来源信息的最终结果
- `2026-05-20` 已完成回归测试与全量测试，覆盖 `chat` / `search` 的流式与非流式行为，`cargo test --quiet` 全绿
- `2026-05-20` 已对照 Hermes Agent 的 Grok / Codex Responses 流式实现，确认正文 token 只来自 `response.output_text.delta`；按当前产品取舍，`grok-cli` 已关闭 `Thinking...` / `Searching X...` 等人类状态提示，默认流式只输出回答正文，避免污染终端、SKILL 和管道捕获结果

### Phase 15. Timeout Policy

目标：按能力类型设置合理默认超时，避免文本长推理被过早断开，同时避免媒体同步请求无意义长等。

- [x] 文本 `chat` / `search` Responses 请求默认超时调整为 `3600` 秒
- [x] `chat --no-stream` / `search --no-stream` / `--json` 与流式文本保持同一个 `3600` 秒默认超时
- [x] `image` 图片生成默认单次 HTTP 超时调整为 `120` 秒
- [x] `tts` 默认单次 HTTP 超时调整为 `120` 秒
- [x] `stt` 默认单次 HTTP 超时调整为 `120` 秒
- [x] `video` 单次 create / poll HTTP 请求保持 `120` 秒上限
- [x] `video` 整体轮询等待默认调整为 `600` 秒
- [x] `video --timeout <SECONDS>` 解释为整体轮询等待上限，允许用户调到 `900` 秒或更高
- [x] 补回归测试锁定文本 / 媒体 / 视频默认超时策略
- [x] 更新命令 help 与 docs timeout 说明

Phase 15 完成标准：

- 文本默认不会因 60 秒客户端超时而断开长推理
- 图片、TTS、STT 默认同步请求不会无意义长等
- 视频不会把整体等待超时误用成每次 HTTP 请求超时
- `cargo test --quiet` 全绿

### Phase 16. SuperGrok Media Capability Completion

目标：把 `image` / `tts` / `stt` 从当前主路径可用推进到更完整的 SuperGrok 媒体能力覆盖，补齐 Hermes Agent 和 xAI 官方文档中已经确认的参数、返回结构和后续扩展入口。

详细任务拆解见 [`docs/project/supergrok-media-capability-plan.md`](./project/supergrok-media-capability-plan.md)。本阶段采用“每补一个能力都必须补测试”的硬门槛。

- [x] Phase 16.1: STT batch completion
  - [x] 补齐 `url`、`format`、`audio_format`、`sample_rate`、`multichannel`、`channels`、`diarize`、`keyterm`、`filler_words`
  - [x] 补齐 `language`、`duration`、`words`、`channels` 等结构化输出
  - [x] 同步补模块级测试、命令级 stub 测试和 `stt` 文档
- [x] Phase 16.2: TTS parameter completion
  - [x] 补齐显式 `output_format`、`sample_rate`、`bit_rate`
  - [x] 补齐 `optimize_streaming_latency`、`text_normalization`、`language=auto`
  - [x] 增加 voice discovery 入口
  - [x] 同步补模块级测试、命令级 stub 测试和 `tts` 文档
- [x] Phase 16.3: Image generation completion
  - [x] 补齐 `count` 和显式 `response_format`
  - [x] 支持多图响应解析和 `images` 输出，同时保留 `image`
  - [x] 同步补模块级测试、命令级 stub 测试和 `image` 文档
- [x] Phase 16.4: Streaming STT
  - [x] 新增实时 STT 协议设计与实验入口
  - [x] 补 WebSocket 参数构造和事件解析测试
  - [~] 暂缓真实协议成功流 mock WebSocket 测试和实时发送细节，不作为当前发布阻塞项
- [x] Phase 16.5: Image editing
  - [x] 新增 image edit / multi-image edit 独立命令设计
  - [x] 补请求构造、输入数量校验和命令级 stub 测试
- [x] Phase 16.6: Imagine video follow-up
  - [x] 核对 video editing / extension 官方接口
  - [x] 拆分后续任务或新增独立计划
  - [x] 实现 `video-edit`
  - [x] 实现 `video-extend`

Phase 16 完成标准：

- 每个完成的能力都有对应模块级测试
- 每个完成的用户可见参数都有命令级 stub 测试
- 每个完成的用户可见行为都有 docs 更新
- 保持 `image.image`、`tts.file_path`、`stt.transcript` 等既有 JSON 主字段兼容
- 每个子阶段完成前对应 targeted tests 通过，整个阶段完成前 `cargo test --quiet` 全绿

本轮补充：

- `2026-05-21` 已完成 STT batch 参数补齐，提交 `9183da8 feat: complete xAI STT batch parameters`。
- `2026-05-21` 已完成 TTS 参数补齐和 voice discovery，提交 `150d853 feat: complete xAI TTS parameters`。
- `2026-05-21` 已完成 image generation 多图与 response format，提交 `ba16f4a feat: complete xAI image generation parameters`。
- `2026-05-21` 已新增实验性 `stt-stream`，提交 `8954596 feat: add experimental streaming STT`。
- `2026-05-21` 已新增 `image-edit`，提交 `10d0268 feat: add xAI image editing command`。
- `2026-05-21` 已拆出 `video-edit` / `video-extend` 后续计划，提交 `380781b docs: plan xAI video editing follow-up`。
- `2026-05-21` 已新增 `video-edit`，提交 `c4d06c8 feat: add xAI video editing command`。
- `2026-05-21` 已新增 `video-extend`，补齐请求构造、duration clamp、命令级 stub 测试和文档。

## 推荐执行顺序

1. 先冻结 `Cargo.toml`、JSON 信封、错误码映射，避免后面边写边改协议。
2. 先完成工程骨架和 `AppContext`，让后续模块共享同一套运行时、HTTP client、状态路径和日志。
3. 先把 `state` / `status` 做稳，再进入登录与刷新流程。
4. 登录层稳定后，先做 `search`，把真实请求路径打通。
5. 在统一 upstream client 上继续扩展 media、chat、stream。
6. 最后再做 SKILL 集成层和验收资料，而不是一开始就把精力放在交互包装上。

## 关键依赖关系

- `Phase 1` 依赖 `Phase 0` 的冻结结果
- `Phase 2` 依赖 `Phase 1` 的工程骨架和共享上下文
- `Phase 3` 依赖 `Phase 2` 的状态文件与错误模型
- `Phase 4` 依赖 `Phase 3` 的有效 OAuth 流程
- `Phase 5` 依赖 `Phase 4` 的共享 upstream client
- `Phase 6` 依赖 `Phase 4` 和 `Phase 5` 的请求执行基础设施
- `Phase 7` 依赖前面各阶段至少完成主路径闭环
- `Phase 8` 依赖 `Phase 3` 的共享 runtime credentials，以及 `Phase 4` / `Phase 6` 的真实请求执行路径，才能拿到本地 usage 与 recent rate-limit snapshot

## 第一版必须跑通的闭环

下面这组能力是第一版“能用”的最低标准：

1. `status`
2. `login`
3. `refresh`
4. `state`
5. `search`
6. `usage`

说明：

- `chat` 虽然重要，但如果流式协议细节还需要继续观察，可以放在第二个可运行里程碑完成
- `image`、`video`、`tts`、`stt` 可以在第一版闭环之后快速并入
- `usage` 作为本地 session 统计入口，应在主能力闭环稳定后尽快补齐

## 当前建议的立即行动

当前工程已经完成第一轮可运行实现，下一步应围绕发布准备和真实用户体验继续推进：

- [x] 创建 `grok-cli/` 工程目录
- [x] 按文档落地 `Cargo.toml`
- [x] 建立 `src/main.rs` / `src/app.rs` / `src/cli.rs` / `src/args.rs`
- [x] 建立 `src/error.rs` / `src/output.rs`
- [x] 建立 `src/state/model.rs` / `src/state/storage.rs`
- [x] 跑通 `state`、`status`
- [x] 替换 README / docs 里的 GitHub `<owner>` 占位符
- [x] 增加 GitHub Release workflow，并在 release guide 保留手动 release 兜底流程
- [x] 确认是否保留 `publish = false`，或准备 crates.io 发布元数据
- [x] 确认 GitHub owner 是否从 `Moore-developers` 迁移为 `Moore`，并同步 remote、Cargo metadata、README、release docs。当前 `Moore` 用户名已被占用，继续使用 `Moore-developers`。
- [x] 补开源协作文件：`CHANGELOG.md`、`CONTRIBUTING.md`、`SECURITY.md`
- [x] 补 GitHub issue / PR templates

## 验收标准

- `grok-cli` 能在目标工具链上编译通过
- 所有核心命令支持 `--json`
- 成功与失败都遵循统一 JSON 信封
- 状态文件可以稳定读写，并能正确脱敏展示
- OAuth 能区分缺失、可刷新、需重登、entitlement 被拒绝
- `chat`、`search`、`image`、`video`、`tts`、`stt` 复用同一套状态、认证和上游请求基础设施
- 后续 SKILL 可以把 CLI 当作稳定脚本执行层，而不是临时实验工具
