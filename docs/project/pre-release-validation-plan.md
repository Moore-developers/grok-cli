# Pre-release Validation Plan

这份文档用于最终确认 `grok-cli` 在公开推广前还需要完成的验收、SKILL 补全和极简性能分析。当前不直接执行真实测试；先确认方案，再按顺序逐项推进。

## 目标

- 确认新增媒体能力在真实 xAI / SuperGrok OAuth 环境下可用。
- 把仓库内置 `grok-cli` skill 做成用户主入口：基础能力在主 `SKILL.md` 里直接可用，高级参数和完整命令面通过关联文件按需加载。
- 建立一套极简性能分析策略，只记录安装后的大概 CPU、内存和体积大小。
- 保持首版发布策略为 SKILL-first / source-first，不发布预构建二进制。

## 不做的事

- 暂不恢复 GitHub Release binary workflow。
- 暂不做 `stt-stream` 深层 WebSocket mock / 分块发送测试。
- 暂不发布 crates.io / Homebrew / winget / Scoop。
- 暂不把真实 OAuth token、session db、媒体文件或转写内容提交进仓库。

## 阶段 0：发布与安装闭环

目的：先验证用户能通过当前公开路径拿到 CLI 和 SKILL。

任务：

- [ ] 从干净环境验证 `cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked`。
- [ ] 验证安装后 `grok-cli --version`、`grok-cli --help`、`grok-cli status --json`。
- [ ] 验证 `skills/grok-cli` 可以复制到 `~/.agents/skills/grok-cli` 或 `~/.codex/skills/grok-cli`。
- [ ] 验证 skill 在 CLI 缺失时能走安装检查路径。
- [ ] 验证 skill 在 Cargo 缺失时能给出清晰提示，而不是继续失败。
- [ ] 验证 skill 安装 CLI 后能恢复原始 Grok 任务。

验收标准：

- 用户无需下载预构建二进制。
- 用户如果已有 Rust/Cargo，可以通过 skill 或 `cargo install --git` 完成安装。
- 用户如果没有 Rust/Cargo，skill 能清楚说明前置条件。

建议提交：

- `docs: add pre-release validation plan`
- 如果后续改 skill 安装逻辑，再单独提交 `docs: harden grok-cli skill install flow`

## 阶段 1：真实 OAuth 回归

目的：确认公开安装路径下 OAuth 主链路仍然稳定。

任务：

- [ ] `grok-cli status --json`：记录未登录 / 已登录状态。
- [ ] `grok-cli login`：完成真实浏览器 OAuth。
- [ ] `grok-cli status --json`：确认登录后状态可读。
- [ ] `grok-cli refresh --json`：确认 refresh 可用。
- [ ] `grok-cli state --json`：确认脱敏输出不泄露 token。
- [ ] 记录错误恢复路径：如果 login callback 未命中，确认 hidden `exchange-code` 救援路径仍可用。

验收标准：

- 登录后所有能力命令能复用同一份 OAuth 状态。
- `auth_relogin_required` 与 `xai_oauth_tier_denied` 能被区分。
- `state --json` 不输出 access token / refresh token。

## 阶段 2：新增媒体能力真实测试

目的：按能力逐个验证，不一次性开太多真实请求，便于定位失败点和控制成本。

### 测试顺序

1. `tts` 生成测试音频，作为后续 `stt` 输入。
2. `stt` 转写上一步音频，验证音频闭环。
3. `image` 单图 URL 输出。
4. `image` 多图 URL 输出。
5. `image` 多图 b64 落盘。
6. `image-edit` 单图编辑。
7. `image-edit` 多图编辑。
8. `video` text-to-video。
9. `video` image-to-video。
10. `video` reference image video。
11. `video-edit` 已有视频编辑。
12. `video-extend` 已有视频扩展。
13. `usage --json` 验证真实请求后的分类统计。

### 输入资产准备

- [ ] 准备一张小尺寸本地图片用于 `image-edit`。
- [ ] 准备 2-3 张本地或远程参考图用于多图编辑。
- [ ] 准备一个可公开访问的 `.mp4` URL，用于 `video-edit` 和 `video-extend`。
- [ ] 准备一个可公开访问的图片 URL，用于 image-to-video。
- [ ] 准备输出目录 `.tmp/media-real-validation/`，并确认 `.tmp/` 不会进入 git。

### 每个能力的记录格式

每条真实测试都记录：

- 命令。
- 是否使用 `--json`。
- 请求参数。
- 成功 / 失败。
- 返回主字段，例如 `image`、`images`、`video`、`file_path`、`transcript`。
- 是否发生 token refresh。
- 是否写入 usage。
- 如果失败，记录错误码和是否可重试。

### 验收标准

- 每个新增媒体能力至少真实成功一次。
- `image` / `video` / `tts` / `stt` 的稳定主字段存在。
- `video-edit` 返回 `modality: "edit"`。
- `video-extend` 返回 `modality: "extension"`。
- 媒体命令在 access token 接近过期时仍能先 refresh 再请求。
- 真实验证结果写入文档，但不提交真实媒体文件和敏感输出。

建议提交：

- `docs: record media real-world validation`

## 阶段 3：SKILL 补全设计

目的：让 `skills/grok-cli/SKILL.md` 提供基础能力入口，把高级参数和完整命令面放到关联文件中，避免主 SKILL 过长。

### 当前缺口

主 SKILL 已覆盖：

- 安装检查
- Cargo 安装
- OAuth status / login
- 基础命令路由
- 基础错误处理

仍需补：

- 完整命令面：`login`、`status`、`refresh`、`logout`、`state`、`model`、`usage`、`chat`、`search`、`image`、`image-edit`、`video`、`video-edit`、`video-extend`、`tts`、`stt`、`stt-stream`。
- 高级参数：chat/search tools、image count / response format / output paths、TTS voice / format / sample rate、STT URL / diarize / keyterm / channels、video image/reference inputs。
- 错误恢复矩阵。
- 输出字段解释。
- 安装与升级策略。

### 建议文件结构

```text
skills/grok-cli/
├─ SKILL.md
└─ references/
   ├─ install-and-auth.md
   ├─ commands-basic.md
   ├─ commands-media.md
   ├─ commands-advanced.md
   ├─ errors.md
   └─ outputs.md
```

### 主 SKILL 保留内容

- 何时触发。
- 核心工作流。
- 安装检查。
- OAuth 检查。
- 常见基础命令。
- 何时读取哪个 reference 文件。

### Reference 文件职责

- `install-and-auth.md`：Cargo 安装、tag 安装、升级、status/login/refresh/logout/state。
- `commands-basic.md`：chat、search、usage、model 的基础用法。
- `commands-media.md`：image、image-edit、video、video-edit、video-extend、tts、stt 的常用用法。
- `commands-advanced.md`：所有高级参数和组合规则。
- `errors.md`：错误码、恢复策略、entitlement 与 relogin 区分。
- `outputs.md`：JSON 输出主字段和稳定契约。

### SKILL 测试建议

- [ ] 测试“用户说用 Grok 总结”，skill 能选择 `chat --json`。
- [ ] 测试“用户说用 Grok 搜 X”，skill 能选择 `search --json`。
- [ ] 测试“用户说生成图片/编辑图片/生成视频/扩展视频”，skill 能选择正确媒体命令。
- [ ] 测试 CLI 缺失时，skill 先检查 Cargo，再安装 CLI。
- [ ] 测试 auth 缺失时，skill 先登录再恢复原始任务。

建议提交：

- `docs: expand grok-cli skill references`

## 阶段 4：极简性能分析

目的：拿到一个大概资源指标，确认安装后的 `grok-cli` 没有明显异常。这里不做深入诊断，不追求精确性能调优，也不把上游网络等待当作本地性能问题。

### 指标

- CPU：记录 user / system CPU time 或 CPU percent 的大概值。
- 内存：记录 peak RSS / maximum resident set size。
- 体积大小：记录安装后二进制大小、关键媒体输出文件大小；如果 usage/session db 明显增长，再记录最终大小。

### 命令分组

- 本地命令：`grok-cli --version`、`grok-cli status --json`。
- 文本命令：从真实测试里选 1 条 `chat --json` 或 `search --json`。
- 媒体命令：复用真实媒体测试中的 `image`、`tts`、`stt`、`video` 各 1 条即可。
- 安装产物：记录 `grok-cli` 二进制体积。

### 推荐工具

- macOS：`/usr/bin/time -l`。
- Linux：`/usr/bin/time -v`。
- 体积：`ls -lh` 或 `du -h`。
- 输出目录：`.tmp/perf/`，最终文档只写摘要。

### 采样策略

- 本地轻量命令最多跑 3 次，看数值是否稳定。
- 真实上游请求只顺手记录 1 次，不额外制造请求成本。
- 媒体命令只记录最终输出文件大小，不分析生成过程。
- 如果指标没有明显异常，不进入代码优化。

### 报告格式

```text
Command:
Platform:
Runs:
CPU:
Peak memory:
Binary size:
Output size:
Notes:
```

### 验收标准

- 有一张简短表格，列出代表命令的 CPU、内存和体积大小。
- 能判断是否存在明显异常，例如内存峰值过高、CPU 占用异常、二进制或媒体输出体积异常。
- 没有明显异常就不继续展开性能优化。

建议提交：

- `docs: add grok-cli perf smoke baseline`

## 阶段 5：安全与隐私护栏

目的：真实测试前后确认不会把敏感信息带进仓库。

任务：

- [ ] 确认 `.tmp/`、`target/`、日志文件不会进入 git。
- [ ] 确认真实 `auth.json` 不在仓库内。
- [ ] 确认 session db 不在仓库内。
- [ ] 确认真实媒体输出和转写文本只放 `.tmp/`。
- [ ] 真实结果写文档时只保留脱敏字段、成功状态、错误码、CPU / 内存 / 体积摘要。

验收标准：

- `git status --short` 不出现敏感文件。
- 文档不包含 token、账号标识、私密媒体 URL 或未脱敏转写内容。

## 推荐执行顺序

1. 阶段 0：发布与安装闭环。
2. 阶段 1：真实 OAuth 回归。
3. 阶段 3：SKILL 补全设计与文件拆分。
4. 阶段 2：新增媒体能力真实测试。
5. 阶段 4：极简性能分析。
6. 阶段 5：安全与隐私护栏，贯穿每个阶段，最后再集中检查一次。

这个顺序的理由：

- 先确保用户能安装和登录，再测试能力。
- 先补 SKILL 路由和 reference 文件，再用真实测试结果反哺 SKILL。
- 极简性能分析放在真实测试后，能顺手复用同一批命令和输入资产。

## 最终确认点

请确认以下选择：

- 是否接受首版继续 SKILL-first / source-first，不发布预构建二进制。
- 是否按上述顺序先做安装/OAuth，再补 SKILL references，再做真实媒体测试。
- `stt-stream` 是否继续保持实验入口，只做基础文档和现有测试，不进入深层 mock。
- 极简性能分析是否只记录 CPU、内存和体积大小，不做深入诊断。
