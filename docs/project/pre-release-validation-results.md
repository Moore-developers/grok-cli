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

## 2026-05-21 阶段 3：SKILL 补全

状态：通过。

### 已完成项目

- `skills/grok-cli/SKILL.md` 增加关键命令面检查，不再只依赖 `grok-cli --version`。
- 新增 `skills/grok-cli/references/`，把完整能力面拆到关联文件：
  - `install-and-auth.md`
  - `commands-basic.md`
  - `commands-media.md`
  - `commands-advanced.md`
  - `errors.md`
  - `outputs.md`
- 增加 `bundled_skill_requires_command_surface_check` 回归测试，防止 skill 漏掉 reference 指针或关键命令检查。

### 已验证项目

- `cargo test --test contract_regressions`：通过，8 个测试全部通过。

## 2026-05-21 阶段 2：新增媒体能力真实测试

状态：通过。

### 测试环境

- 使用隔离安装的 `v0.1.0` 二进制执行真实请求。
- OAuth 状态：已登录，测试期间没有触发 `auth_relogin_required`。
- 临时输出目录：`.tmp/media-real-validation/`，不会进入 git。

### 已验证项目

- `tts --json`：通过，生成 MP3 文件，返回 `file_path` 和 `media_tag`。
- `stt --json`：通过，成功转写上一步 TTS 音频，返回 `transcript`、`language`、`duration` 和 `words`。
- `image --json` 单图 URL：通过，返回 `image` 和 `images`。
- `image --json --count 2 --response-format url`：通过，`images` 返回 2 个 URL。
- `image --json --count 2 --output-dir ...`：通过，写入 2 个本地 PNG 文件。
- `image-edit --json` 单图编辑：通过，返回 `image` 和 `images`。
- `image-edit --json` 多图编辑：通过，`extra.input_count=2`。
- `video --json` text-to-video：通过，返回 `video`、`duration`、`extra.request_id`，`modality=text`。
- `video --json --image-url ...` image-to-video：通过，返回 `video` 和 `modality=image`。
- `video --json --reference-image-url ...` reference image video：通过，返回 `video`；当前输出 `modality=image`，不是单独的 `reference`。
- `video-edit --json`：通过，返回 `video` 和 `modality=edit`。
- `video-extend --json`：通过，返回 `video` 和 `modality=extension`。
- `usage --json`：通过，真实请求后 image/audio/video 分类计数均可读。

### 体积摘要

- TTS MP3：约 `50K`。
- 多图落盘 PNG：约 `226K` 和 `210K`。
- `.tmp/media-real-validation/` 总大小：约 `492K`。
- `session.db`：约 `56K`。

### 观察

- reference image video 当前返回 `modality=image`。这不阻塞首版发布，但如果希望输出更精细，可以后续把 reference image video 标记成独立 modality。
- 真实媒体文件、转写文本和 URL 都未写入仓库；本文只保留脱敏摘要。

## 2026-05-21 阶段 4：极简性能分析

状态：通过。

### 指标摘要

| 命令 | CPU | Peak memory | Binary size | Output size | 备注 |
| --- | --- | --- | --- | --- | --- |
| `grok-cli --version` | 约 `0.00 user / 0.00 sys` | 约 `7.3 MB` RSS | 约 `7.9M` | 无 | 本地轻量命令 |
| `grok-cli status --json` | 约 `0.00 user / 0.00 sys` | 约 `8.6 MB` RSS | 约 `7.9M` | JSON 状态 | 本地 auth 状态读取 |
| `grok-cli usage --json` | 约 `0.00 user / 0.00 sys` | 约 `10.2 MB` RSS | 约 `7.9M` | JSON usage | 本地 SQLite 读取 |
| 媒体真实测试输出 | 不单独分析 | 不单独分析 | 约 `7.9M` | 约 `492K` | 只记录文件体积 |

### 结论

- 基础命令 CPU 和内存没有明显异常。
- 二进制体积约 `7.9M`，首版可接受。
- 媒体输出体积在本次短样例下没有异常膨胀。
- 不进入深入性能诊断。

## 2026-05-21 阶段 5：安全与隐私护栏

状态：通过。

### 已验证项目

- `.tmp/`、`target/` 和 `*.log` 已在 `.gitignore` 中忽略。
- 真实 OAuth `auth.json` 不在仓库内。
- 真实 `session.db` 不在仓库内。
- 真实媒体输出和转写文件只保存在 `.tmp/`。
- 文档只保留成功状态、错误码/观察和 CPU / 内存 / 体积摘要。
