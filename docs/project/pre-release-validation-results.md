# Pre-release Validation Results

这份文档记录发布前验证的脱敏结果。敏感 token、账号标识、真实媒体内容和私密 URL 不写入仓库；临时输出只放在 `.tmp/`。

## 2026-05-21 阶段 0：发布与安装闭环

状态：通过，发现 1 个需要修复的 skill 安装检查问题。

### 验证环境

- 平台：macOS，本机验证。
- Rust/Cargo：Cargo 可用，版本为 `1.92.0`。
- 安装方式：隔离目录执行 `cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked`。
- 安装来源：GitHub tag `v0.1.0`，解析到提交 `d95b84a`。
- 安装结果：成功。
- 编译耗时：约 5 分 55 秒，仅作为安装观察，不作为性能目标。
- 安装后二进制大小：约 `7.9M`。

### 已验证项目

- `grok-cli --version`：通过，输出 `grok-cli 0.1.0`。
- `grok-cli --help`：通过，公开命令包含 `login`、`status`、`refresh`、`logout`、`state`、`model`、`usage`、`chat`、`search`、`image`、`image-edit`、`video`、`video-edit`、`video-extend`、`tts`、`stt`、`stt-stream`。
- `grok-cli status --json`：通过，JSON 可读，当前 OAuth 状态为已登录。
- `skills/grok-cli` 复制到 agent/codex skill 目录结构：通过，临时目录验证 `SKILL.md` 可以完整复制。
- `.tmp/` 已在 `.gitignore` 中忽略：通过。

### 发现的问题

- 本机全局已有的 `grok-cli 0.1.0` 是旧安装，缺少 `image-edit`、`video-edit`、`video-extend`、`stt-stream` 等命令。
- 因为版本号同为 `0.1.0`，skill 不能只检查 `grok-cli --version`。需要额外检查关键命令是否存在；如果缺失，应引导重新执行 `cargo install --git ... --tag v0.1.0 --locked --force`。

### 后续动作

- 在 `skills/grok-cli` 中补充命令面检查。
- 为 skill 文档增加轻量测试，确保关键命令和重装提示不会遗漏。

## 2026-05-21 阶段 1：OAuth 状态回归

状态：部分通过。浏览器 `login` 主流程未重新触发，因为当前本机已经有可用 OAuth 状态。

### 已验证项目

- `grok-cli status --json`：通过，已登录，`relogin_required=false`，`entitlement_denied=false`。
- `grok-cli refresh --json`：通过，refresh 成功并更新 `last_refresh`。
- `grok-cli state --json`：通过，token 字段为脱敏显示，没有输出完整 access token / refresh token。
- `grok-cli model --json`：通过，默认文本模型可读。
- `grok-cli usage --json`：通过，本地 usage 数据可读。

### 未完成项目

- `grok-cli login` 真实浏览器重新登录：暂未执行，避免在已有可用 session 时强制打断当前状态。
- hidden `exchange-code` 救援路径：暂未执行，留到需要重新登录或专门认证回归时验证。

### 后续动作

- 如果后续真实媒体测试遇到 `auth_relogin_required`，再执行 `grok-cli login` 并记录登录闭环。
- 真实测试结果仍只写脱敏状态、错误码和必要摘要。
