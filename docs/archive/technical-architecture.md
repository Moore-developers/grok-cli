# Grok SKILL 技术架构文档

## 1. 目标

构建一套以 OAuth 为主的 Grok 能力路由架构，让用户可以通过自己的 xAI OAuth 使用 Grok 的完整能力，并且在行为上尽量参考 Hermes 当前对 Grok 的接入方式。

这套架构的目标不是复刻 Hermes 的全部产品形态，而是把 Hermes 已验证可行的 Grok OAuth 能力，拆成一个可复用、可维护、可扩展的 SKILL 路由系统。

## 2. 设计原则

- OAuth-first
- 能力路由优先于命令拼接
- 状态持久化优先于重复登录
- 失败要可分类，不能只返回“失败”
- 命令行层只负责确定性执行，不负责自然语言理解
- 兼容 Hermes 当前行为，但不把 Hermes 内部实现暴露给用户

## 3. 总体分层

```text
用户
  |
  v
Grok SKILL
  |
  | 负责意图识别、任务分类、是否先认证、如何恢复任务
  v
grok-cli
  |
  | 负责 OAuth、状态文件、能力调用、代理启动、结构化输出
  v
xAI / Grok 能力面
```

### 3.1 SKILL 层

SKILL 是路由入口，负责理解用户“想用 Grok 干什么”：

- 聊天 / 推理
- X 搜索
- 图片生成
- 视频生成
- TTS
- STT
- OpenAI-compatible 代理

SKILL 不直接处理 HTTP 细节，也不直接管理 token。

### 3.2 `grok-cli` 层

`grok-cli` 是执行层，负责：

- 检查 OAuth 状态
- 生成 / 刷新 / 保存 token
- 维护本地 session usage 与成本统计
- 输出预留的 `account_limits` 结构
- 调用 xAI 接口
- 启动本地 proxy
- 输出统一 JSON

### 3.3 xAI 能力层

这是最终的能力来源，包含：

- `codex_responses`
- `chat_completions`
- `completions`
- `embeddings`
- `models`
- `x_search`
- `images/generations`
- `videos/generations`
- `tts`
- `stt`

## 4. 核心组件

### 4.1 意图路由器

职责：

- 识别用户是否在请求 Grok
- 识别请求属于哪种 Grok 能力
- 决定是否要先做认证

输入：

- 用户自然语言
- 当前上下文
- 当前 OAuth 状态

输出：

- 路由类型
- 需要调用的 `grok-cli` 命令
- 是否先认证

### 4.2 OAuth 状态管理器

职责：

- 读取本地状态文件
- 判断 token 是否存在
- 判断 token 是否可复用
- 判断是否需要 refresh
- 判断是否属于 relogin / entitlement denied

状态文件建议为：

```text
~/.grok-cli/auth.json
```

### 4.3 OAuth Auth Runtime

职责：

- 发起 PKCE 登录
- 打开浏览器 / 支持 manual-paste
- 启动本地 HTTP callback runtime，而不是只做一次性 socket 监听
- 接收 callback，并返回可诊断的成功页 / 失败页
- 在 loopback 未命中时，自动切换或显式进入 manual-paste fallback
- 交换 token
- 刷新 token
- 写回状态文件

设计要求：

- 这层需要尽量贴近 Hermes 当前的认证接收分层，而不是把所有逻辑塞进一个 `auth login` 函数里
- callback 接收、manual-paste 解析、token exchange、refresh 不应共享一段脆弱的“读第一段 TCP 数据”实现
- 这层最终应成为 `task`、`proxy` 和后续共享 runtime credential resolver 的前置基础设施
- 对 `token exchange` / `refresh` 的 HTTP 请求需要具备基础稳态能力：显式 timeout、明确的 accept / content-type、轻量 transport retry、失败上下文回写
- 真实环境里要注意浏览器 / curl 与 Rust HTTP 栈可能走出不同的网络路径；`auth.x.ai` 这类 OAuth endpoint 若在当前环境下出现 Rust `connect` 失败但浏览器成功，需要优先排查地址族选择（IPv6 / IPv4）、TLS 栈差异与 Happy Eyeballs 行为，而不是先怀疑 OAuth 参数本身

### 4.4 Runtime Credential Resolver

职责：

- 读取认证状态并解析当前可用 runtime credentials
- 统一输出 `provider` / `base_url` / bearer / expiry / source
- 判断 access token 是否可直接复用
- 判断是否需要 refresh
- 在 refresh 成功后回写状态
- 为 `task` 与 `proxy` 提供同一套运行时凭据视图

这层的目标是对齐 Hermes 的共享 runtime resolver 设计，避免：

- `auth`、`task`、`proxy` 各自维护一套 bearer 解析逻辑
- 后续 `codex_responses` 主运行时和认证刷新逻辑彼此脱节

### 4.5 能力执行器

职责：

- `task chat`
- `task x-search`
- `task image-gen`
- `task video-gen`
- `task tts`
- `task stt`

这层只处理结构化参数，不做自然语言解析。

### 4.6 Usage / Quota 聚合器

职责：

- 聚合本地 session history
- 汇总 input / output tokens 与估算成本
- 保留最近一次 rate-limit headers 快照
- 对外输出统一 `usage` JSON 结构
- 保留 `account_limits` 字段位，当前不对 xAI 发起 live quota / entitlements 探测

设计要求：

- 本地统计是主路径
- `account_limits` 当前为保留位，未实现时必须稳定返回 disabled 状态
- 命令层不直接写死 xAI quota endpoint
- provider-specific quota 读取继续保留扩展点，但当前默认禁用

### 4.7 Proxy 适配器

职责：

- 启动本地 OpenAI-compatible HTTP 服务
- 将客户端请求转发到 xAI
- 注入 OAuth bearer
- 保留 SSE 流
- 仅允许 Hermes 当前支持的路径

允许路径：

- `/v1/responses`
- `/v1/chat/completions`
- `/v1/completions`
- `/v1/embeddings`
- `/v1/models`

## 5. 项目结构与依赖策略

### 5.1 结构策略

本项目不建议走“先单文件最小化、后续再重构”的路线，而应直接按目标结构建仓。

原因：

- 这个项目从第一天起就同时包含 OAuth、状态持久化、真实 HTTP 调用、流式输出、代理转发五类复杂度
- `auth`、`task`、`proxy` 三条线的边界天然清晰，提前拆开比后补拆分成本更低
- SKILL 路由层最终依赖 `grok-cli` 的稳定命令面和错误模型，过渡结构越多，后续适配成本越高
- 如果一开始就知道目标是长期维护的能力系统，那么目录结构、错误模型、输出协议、状态 schema 应该一次设计到位

因此，仓库初始形态就建议采用目标结构，而不是把“目标结构”作为后续重构事项。

### 5.2 目标项目结构

推荐从第一版开始就使用下面的结构，方便维护 OAuth、能力调用和 proxy 三条线：

```text
grok-cli/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── .sample/
│   └── auth.json
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── cli.rs
│   ├── args.rs
│   ├── error.rs
│   ├── output.rs
│   ├── state/
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   └── storage.rs
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── runtime.rs
│   │   ├── callback.rs
│   │   ├── login.rs
│   │   ├── resolver.rs
│   │   ├── refresh.rs
│   │   └── pkce.rs
│   ├── task/
│   │   ├── mod.rs
│   │   ├── chat.rs
│   │   ├── search.rs
│   │   ├── image.rs
│   │   ├── video.rs
│   │   └── audio.rs
│   ├── usage/
│   │   ├── mod.rs
│   │   ├── command.rs
│   │   ├── model.rs
│   │   ├── pricing.rs
│   │   ├── tracker.rs
│   │   └── sqlite.rs
│   ├── providers/
│   │   ├── mod.rs
│   │   ├── base.rs
│   │   └── xai/
│   │       ├── mod.rs
│   │       └── client.rs
│   ├── proxy/
│   │   ├── mod.rs
│   │   ├── server.rs
│   │   ├── routes.rs
│   │   └── upstream.rs
│   └── debug/
│       ├── mod.rs
│       └── observations.rs
└── tests/
    ├── auth_status.rs
    ├── state_validate.rs
    └── cli_smoke.rs
```

### 5.3 模块职责建议

- `main.rs`：程序入口，只做启动、退出码转换和顶层错误兜底
- `app.rs`：应用装配、共享依赖注入、运行上下文初始化
- `cli.rs`：顶层命令分发
- `args.rs`：命令和参数解析
- `output.rs`：统一 JSON 输出、流式事件输出
- `error.rs`：统一错误码、错误信封、退出码映射
- `state/`：认证状态文件读写、schema 校验、路径管理、脱敏展示
- `auth/`：OAuth auth runtime、callback 接收、登录、刷新、PKCE、授权码交换、runtime credentials 解析
- `task/`：聊天、搜索、图片、视频、TTS、STT 的任务执行与上游参数适配
- `usage/`：session usage 聚合、SQLite 历史、成本估算、rate-limit snapshot、`usage` 命令输出
- `providers/`：provider-specific 扩展点；当前 `xai` account limits 返回 disabled 状态
- `proxy/`：OpenAI-compatible 本地代理服务、白名单路由、SSE 透传、上游转发
- `debug/`：Hermes 行为观察、参数回放、协议差异记录

### 5.4 依赖包策略

本项目建议直接采用“核心依赖一次选型到位”的策略，而不是先空依赖、后续按报错补装。

原因：

- 这里的复杂度不是算法复杂度，而是协议复杂度和系统边界复杂度
- OAuth、JSON、HTTP、SSE、proxy 都属于成熟问题域，使用成熟 crate 比手写基础设施更稳定
- 统一的类型系统、错误模型和异步运行时越早确定，越能避免后续跨模块返工
- 这个项目的目标是长期维护的能力系统，不是一次性脚本

工具链建议：

- Rust stable 最新版
- 在 `Cargo.toml` 中显式声明 `rust-version`
- 不以 `rustc 1.68.1` 作为设计上限；如果当前环境受限，应优先升级工具链，而不是为了兼容旧工具链削弱架构

推荐依赖分层如下：

| crate | 用途 | 角色 |
| --- | --- | --- |
| `clap` | 命令行参数解析 | 核心基础依赖 |
| `serde` | 结构化数据模型 | 核心基础依赖 |
| `serde_json` | JSON 读写 | 核心基础依赖 |
| `thiserror` | 错误类型定义 | 核心基础依赖 |
| `tokio` | 异步运行时 | 核心基础依赖 |
| `reqwest` | xAI HTTP 调用 | 核心基础依赖 |
| `url` | 授权 URL / 回调 URL 处理 | OAuth 基础依赖 |
| `base64` | PKCE / token 编码辅助 | OAuth 基础依赖 |
| `sha2` | PKCE challenge 计算 | OAuth 基础依赖 |
| `rand` | PKCE verifier / state 生成 | OAuth 基础依赖 |
| `percent-encoding` | 查询参数编码 | OAuth 辅助依赖 |
| `axum` | 本地 proxy 服务 | 服务层核心依赖 |
| `tower-http` | HTTP 中间件能力 | 服务层辅助依赖 |
| `tracing` | 结构化日志 | 观测基础依赖 |
| `tracing-subscriber` | 日志输出与过滤 | 观测基础依赖 |
| `chrono` 或 `time` | 时间字段处理 | 状态与日志辅助依赖 |
| `uuid` | 请求追踪 / 调试标识 | 调试辅助依赖 |

依赖原则：

- 不是“能不用就不用”，而是“已知一定会用、且边界清晰的基础设施依赖，第一版就纳入”
- 优先选择 Rust 生态中长期稳定、维护活跃、文档成熟的主流 crate
- 初版就固定核心技术栈：`clap + serde + thiserror + tokio + reqwest + axum + tracing`
- 对外暴露的 JSON schema、错误码、代理路径比内部实现更需要稳定，因此相关依赖应尽早定型
- 版本策略建议使用稳定主线版本，并通过 `Cargo.lock` 固定解析结果；升级时集中评估，不做零散漂移

### 5.5 建议的初始依赖集合

第一版 `Cargo.toml` 建议就包含以下依赖集合：

- CLI：`clap`
- 数据模型：`serde`、`serde_json`
- 错误：`thiserror`
- 异步与 HTTP：`tokio`、`reqwest`
- OAuth 辅助：`url`、`base64`、`sha2`、`rand`、`percent-encoding`
- 代理服务：`axum`、`tower-http`
- 观测：`tracing`、`tracing-subscriber`
- 时间与调试：`chrono` 或 `time`、`uuid`

这样做的目标不是“堆依赖”，而是从第一版开始就让项目结构、运行时模型、错误模型、协议模型保持一致。

### 5.6 可直接使用的 `Cargo.toml` 建议稿

下面这版不是示意代码，而是可直接作为第一版起点的依赖建议。

版本说明：

- 以下版本按文档修订时的 crates.io 稳定版本整理
- 实际落地时建议先执行一次 `cargo check`
- 如果团队希望降低升级频率，可以保留这些主版本，并通过 `Cargo.lock` 锁定解析结果

```toml
[package]
name = "grok-cli"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "MIT"
publish = false

[dependencies]
clap = { version = "4.6.1", features = ["derive", "env", "string"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
thiserror = "2.0.18"
tokio = { version = "1.52.3", features = ["macros", "rt-multi-thread", "signal", "net", "time", "fs"] }
reqwest = { version = "0.13.3", default-features = false, features = ["json", "stream", "http2", "charset", "rustls-tls"] }
url = "2.5.8"
base64 = "0.22.1"
sha2 = "0.11.0"
rand = "0.10.1"
percent-encoding = "2.3.2"
axum = { version = "0.8.9", features = ["http1", "http2", "json", "query", "tokio"] }
tower-http = { version = "0.6.11", features = ["cors", "trace", "request-id", "timeout"] }
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter", "fmt", "json"] }
uuid = { version = "1.23.1", features = ["v4", "serde"] }
time = { version = "0.3.47", features = ["serde", "formatting", "parsing"] }

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

补充建议：

- 如果暂时不需要 JSON 日志，可先保留 `tracing-subscriber` 的 `json` feature，不必移除，因为后续 debug 和自动化采集很可能会用到
- 如果未来 proxy 需要更细的流式转发控制，再评估是否引入 `hyper`，但第一版不建议同时维护 `axum` 和 `hyper` 两套路由入口
- `reqwest` 建议默认走 `rustls-tls`，避免对系统 OpenSSL 产生额外环境依赖

### 5.7 feature 选择建议

这里不只是列依赖，还要明确为什么开这些 feature，避免后续 `Cargo.toml` 越长越失控。

| crate | feature | 建议 | 原因 |
| --- | --- | --- | --- |
| `clap` | `derive` | 开启 | 让命令树和参数结构与 Rust 类型直接对齐 |
| `clap` | `env` | 开启 | 便于代理端口、base URL、调试开关走环境变量 |
| `clap` | `string` | 开启 | 便于自定义字符串默认值和展示 |
| `serde` | `derive` | 开启 | 状态模型、输出模型、错误信封都需要 |
| `tokio` | `macros` | 开启 | `#[tokio::main]` 和测试都需要 |
| `tokio` | `rt-multi-thread` | 开启 | proxy 和上游请求天然适合多线程运行时 |
| `tokio` | `signal` | 开启 | 代理服务优雅退出需要 |
| `tokio` | `net` | 开启 | 本地 callback 与 proxy 监听需要 |
| `tokio` | `time` | 开启 | token 过期判断、超时控制需要 |
| `tokio` | `fs` | 开启 | 状态文件异步读写需要 |
| `reqwest` | `json` | 开启 | xAI JSON 请求和响应解析需要 |
| `reqwest` | `stream` | 开启 | SSE 和流式返回需要 |
| `reqwest` | `http2` | 开启 | 对现代 API 调用更稳妥 |
| `reqwest` | `charset` | 开启 | 提高文本响应兼容性 |
| `reqwest` | `rustls-tls` | 开启 | 降低本地环境 TLS 依赖复杂度 |
| `reqwest` | 默认 feature | 关闭 | 避免引入不必要的系统依赖和隐式能力 |
| `axum` | `http1` | 开启 | 本地客户端兼容性需要 |
| `axum` | `http2` | 开启 | 流式和上游代理场景更稳妥 |
| `axum` | `json` | 开启 | 健康检查、状态接口、错误输出需要 |
| `axum` | `query` | 开启 | callback 参数、调试接口需要 |
| `axum` | `tokio` | 开启 | 与运行时一致 |
| `tower-http` | `cors` | 条件开启 | 如果未来本地 Web 客户端要直接接 proxy，会需要 |
| `tower-http` | `trace` | 开启 | 代理链路排障价值很高 |
| `tower-http` | `request-id` | 开启 | 有助于上下游请求关联 |
| `tower-http` | `timeout` | 开启 | 防止代理请求无边界悬挂 |
| `tracing-subscriber` | `env-filter` | 开启 | 让日志级别通过环境变量切换 |
| `tracing-subscriber` | `fmt` | 开启 | 本地开发默认可读日志 |
| `tracing-subscriber` | `json` | 开启 | 自动化调试与采集更方便 |
| `uuid` | `v4` | 开启 | 生成 request id |
| `uuid` | `serde` | 开启 | 便于写入输出模型和调试记录 |
| `time` | `serde` | 开启 | 时间字段持久化需要 |
| `time` | `formatting` | 开启 | JSON 输出与日志格式化需要 |
| `time` | `parsing` | 开启 | token 过期时间解析需要 |

建议暂不开启的内容：

- `clap` 的 shell completion 相关 feature：等命令面稳定后再加
- `reqwest` 的 `multipart`：如果后续图片、音频上传确实需要，再补
- `tower-http` 的 `compression-*`：本地 proxy 第一版不是性能瓶颈
- `uuid` 的更多版本特性：当前只要 v4 就够

### 5.8 首批文件职责与初始化顺序

第一批文件不应该只是“占位”，而应该能支撑 `auth status`、`state show`、`task x-search`、`proxy start` 这几类最关键路径。

建议首批文件职责如下：

| 文件 | 职责 | 第一版是否必须 |
| --- | --- | --- |
| `src/main.rs` | 进程入口、运行时启动、退出码收敛 | 必须 |
| `src/app.rs` | 组装 `AppContext`、加载配置、创建共享 HTTP client | 必须 |
| `src/cli.rs` | 将解析后的命令分发到各模块 handler | 必须 |
| `src/args.rs` | 定义 CLI 命令树、全局参数、子命令参数 | 必须 |
| `src/error.rs` | 统一错误类型、错误码、退出码映射 | 必须 |
| `src/output.rs` | 统一 JSON 成功/失败信封、流式事件输出 | 必须 |
| `src/state/mod.rs` | 对外暴露状态模块接口 | 必须 |
| `src/state/model.rs` | `AuthState`、脱敏视图、运行时派生状态 | 必须 |
| `src/state/storage.rs` | 状态文件路径解析、读写、校验、原子写入 | 必须 |
| `src/auth/mod.rs` | 对外暴露认证模块接口 | 必须 |
| `src/auth/runtime.rs` | auth runtime 编排、浏览器启动、模式切换、会话生命周期 | 必须 |
| `src/auth/callback.rs` | 本地 HTTP callback runtime、callback 页、manual-paste 解析 | 必须 |
| `src/auth/login.rs` | PKCE 参数构造、authorize URL、authorization code exchange | 必须 |
| `src/auth/resolver.rs` | 共享 runtime credentials 解析、复用/刷新判定 | 必须 |
| `src/auth/refresh.rs` | refresh token 刷新与 relogin 判定 | 必须 |
| `src/auth/pkce.rs` | verifier、challenge、state 生成 | 必须 |
| `src/task/mod.rs` | 对外暴露任务模块接口 | 必须 |
| `src/task/chat.rs` | 聊天与流式响应执行 | 建议第一批就建文件 |
| `src/task/search.rs` | X 搜索任务执行 | 必须 |
| `src/task/image.rs` | 图片生成任务执行 | 建议第一批建空壳 |
| `src/task/video.rs` | 视频生成任务执行 | 建议第一批建空壳 |
| `src/task/audio.rs` | TTS / STT 执行 | 建议第一批建空壳 |
| `src/proxy/mod.rs` | 对外暴露代理模块接口 | 必须 |
| `src/proxy/server.rs` | 启动 axum server、优雅退出 | 必须 |
| `src/proxy/routes.rs` | 白名单路由和 handler 绑定 | 必须 |
| `src/proxy/upstream.rs` | 转发到 xAI、注入 bearer、透传流 | 必须 |
| `src/debug/mod.rs` | debug 命令入口 | 建议 |
| `src/debug/observations.rs` | Hermes 行为记录与参数快照 | 建议 |
| `tests/auth_status.rs` | 认证状态检查用例 | 必须 |
| `tests/state_validate.rs` | 状态文件校验用例 | 必须 |
| `tests/cli_smoke.rs` | 命令行基本冒烟测试 | 必须 |

建议的初始化顺序：

1. `main.rs` 启动 `tokio` 运行时并初始化日志。
2. `args.rs` 解析 CLI 输入，得到强类型命令结构。
3. `app.rs` 根据全局参数和环境变量创建 `AppContext`。
4. `AppContext` 至少包含：
   - `state_path_resolver`
   - `reqwest::Client`
   - `base_url`
   - `output_mode`
   - `clock`
   - `request_id_factory`
5. `cli.rs` 根据命令分发给 `state`、`auth`、`task`、`proxy`、`debug`。
6. 各模块内部统一返回领域错误，由 `main.rs` 最终映射为退出码和 JSON 错误信封。

建议的 `AppContext` 初版职责：

- 不在各模块里重复创建 `reqwest::Client`
- 不在各模块里自行拼接状态文件路径
- 不让 `proxy`、`task`、`auth` 各自维护不同的时间和日志上下文
- 不让 `task`、`proxy`、`auth refresh` 各自重复解析 bearer / refresh 判定
- 为后续注入 mock client、测试时钟、临时状态目录预留接口

### 5.9 第一版落地边界

为了保证这套结构“能用”，而不是“只有目录很漂亮”，第一版建议至少完成以下能力闭环：

1. `auth status`
2. `auth login`
3. `auth refresh`
4. `state path`
5. `state show`
6. `state validate`
7. `task x-search`
8. `proxy start`

其中：

- `task chat` 可以在第一批文件中先落壳，但如果 xAI 的真实流式协议还需要进一步观察，可以放在第二个迭代完成
- `image-gen`、`video-gen`、`tts`、`stt` 可以先保留命令占位与统一错误返回，待接口确认后补全
- `proxy status`、`proxy providers` 可以在 `proxy start` 跑通后补

## 6. 数据流

### 6.1 首次使用

```text
用户说“用 Grok 做 X”
  -> SKILL 识别能力类型
  -> grok-cli auth status
  -> 无有效凭据
  -> grok-cli auth login
  -> 保存 auth.json
  -> grok-cli task ...
  -> 返回结果
```

### 6.2 已登录复用

```text
用户说“继续用 Grok 做 X”
  -> SKILL 识别能力类型
  -> grok-cli auth status
  -> 有效凭据
  -> 直接 grok-cli task ...
  -> 返回结果
```

### 6.3 Token 失效

```text
用户说“用 Grok 做 X”
  -> grok-cli auth status
  -> 发现 token 失效 / 需要 refresh
  -> grok-cli auth refresh
  -> refresh 成功则继续
  -> refresh 失败则引导 relogin
```

### 6.4 代理模式

```text
用户想把 Grok 给别的客户端用
  -> grok-cli proxy start
  -> 本地 OpenAI-compatible 代理启动
  -> 第三方客户端连接本地 proxy
  -> proxy 注入 OAuth bearer
  -> 上游 xAI 返回结果
```

## 7. 模块边界

### 7.1 SKILL 负责

- 自然语言理解
- 任务分类
- 用户交互
- 是否先认证
- 何时恢复原始任务

### 7.2 `grok-cli` 负责

- 状态文件读写
- OAuth 流程
- JSON 输出
- xAI 请求执行
- SSE / proxy 转发

### 7.3 xAI 负责

- 模型推理
- 搜索
- 图像 / 视频 / 音频能力
- 权限和 entitlement 判断

## 8. 状态模型

### 8.1 持久化状态

建议保存：

- `version`
- `provider`
- `auth_mode`
- `base_url`
- `tokens`
- `discovery`
- `redirect_uri`
- `last_refresh`
- `last_auth_error`
- `metadata`

### 8.2 运行时状态

建议在执行时派生这些状态：

- `logged_in`
- `access_token_present`
- `refresh_token_present`
- `access_token_expiring`
- `relogin_required`
- `entitlement_denied`

### 8.3 状态机

```text
未登录
  -> 登录中
  -> 已登录
  -> 可刷新
  -> 需要重新登录
  -> entitlement denied
```

## 9. 命令体系

### 9.1 `auth`

- `auth status`
- `auth login`
- `auth refresh`
- `auth logout`
- `auth print-authorize-url`
- `auth exchange-code`

### 9.2 `state`

- `state show`
- `state path`
- `state validate`

### 9.3 `task`

- `task chat`
- `task x-search`
- `task image-gen`
- `task video-gen`
- `task tts`
- `task stt`

### 9.4 `proxy`

- `proxy start`
- `proxy status`
- `proxy providers`

### 9.5 `debug`

- `debug authorize-params`
- `debug token-request-shape`
- `debug hermes-observation`

## 10. 错误分层

### 10.1 认证类错误

- `auth_missing`
- `auth_expired`
- `auth_refresh_failed`
- `auth_relogin_required`
- `auth_state_mismatch`
- `auth_callback_timeout`
- `auth_token_exchange_failed`

### 10.2 权限类错误

- `xai_oauth_tier_denied`

### 10.3 能力类错误

- `model_capability_mismatch`
- `request_failed`

### 10.4 基础类错误

- `invalid_args`
- `io_error`
- `state_file_missing`
- `state_file_invalid`
- `path_not_allowed`

## 11. 流式与非流式

### 11.1 非流式

用于：

- `task x-search`
- `task image-gen`
- `task video-gen`
- `task tts`
- `task stt`

输出最终结构化 JSON。

### 11.2 流式

用于：

- `task chat --stream`
- `proxy` 转发的 SSE

事件格式参考 Hermes 当前行为，使用 `response.*` 系列事件。

## 12. 安全与约束

- token 必须落盘保存，但输出时必须脱敏
- proxy 只允许白名单路径
- 不默认开放 LAN
- 不把 OAuth 失败静默当作 API key 问题
- entitlement denied 必须与 relogin required 区分

## 13. 实施顺序

1. 先完成目标项目骨架、核心依赖、统一错误模型、统一输出模型
2. 完成 `state/` 与 `auth/`，确保 OAuth 状态、刷新、脱敏展示、错误分类已经定型
3. 完成共享上游客户端层，再接入 `task chat` 与 `task x-search`
4. 在同一套请求执行框架下补齐 `image-gen`、`video-gen`、`tts`、`stt`
5. 完成 `proxy`，复用同一 OAuth 状态和上游转发能力，而不是另起一套实现
6. 最后再让 SKILL 路由层绑定这套稳定的命令面、错误码和恢复逻辑

## 14. 当前结论

这套架构的核心，不是“把 Grok 变成一个命令”，而是“把 Grok 变成一个可被 SKILL 路由、可被 OAuth 复用、可被代理复用的能力系统”。
