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
- 文档归档样本目录：`docs/project/tests/2026-05-21T12-20-32+0800/`，用于保留这轮真实测试引用过的本地样本文件。

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

### 本次真实引用与生成对照表

| 能力 | 中文提示词 | 真实引用输入 | 真实生成输出 |
| --- | --- | --- | --- |
| `tts` | `Pre-release validation sample for Grok CLI.` | 无 | 本地音频 [tts-sample.mp3](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-20-32+0800/tts-sample.mp3) |
| `stt` | 无 | 本地音频 [tts-sample.mp3](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-20-32+0800/tts-sample.mp3) | 转写文本 `Pre-release validation sample for Grok CLI.` |
| `image` 单图 URL | 一个小巧的黄铜机器人举着写有“Grok CLI validation”的牌子，棚拍产品照风格 | 无 | 远程图片 [image-001](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-ddebf8f7-1ffc-44b4-82bb-a4bc209a520c.jpeg) |
| `image` 多图 URL | 两个用于 Grok CLI 验证套件的极简应用图标，干净的矢量风格 | 无 | 远程图片 [image-002](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-128f4cbe-67a5-4e5a-8014-021a827781c7.jpeg)、[image-003](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-153c638c-3599-4cbe-9763-8c3a54bb4e3b.jpeg) |
| `image` 多图本地落盘 | 两个用于 Grok CLI 验证的小型黑白终端吉祥物 | 无 | 本地图片 [image-001.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-20-32+0800/images/image-001.png)、[image-002.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-20-32+0800/images/image-002.png) |
| `image-edit` 单图 | 在角落加一个小的绿色对勾徽标 | 本地图片 [image-001.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-20-32+0800/images/image-001.png) | 远程图片 [image-edit-001](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-9d93d048-61ff-425d-88a6-787c85603d1c.jpeg) |
| `image-edit` 多图 | 把这两个吉祥物融合成一个干净的验证徽章 | 本地图片 [image-001.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-20-32+0800/images/image-001.png)、[image-002.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-20-32+0800/images/image-002.png) | 远程图片 [image-edit-002](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-2e2d5228-2f19-49ae-88c3-2527c64ee440.jpeg) |
| `video` text-to-video | 一个两秒钟的简单动画：一个小黄铜机器人在终端窗口旁挥手 | 无 | 远程视频 [video-001](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-e16c4b37-d3a7-4dc0-932f-c62610ed5b3e.mp4) |
| `video` image-to-video | 让这个验证吉祥物原地轻轻挥手 | 远程图片 [image-001](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-ddebf8f7-1ffc-44b4-82bb-a4bc209a520c.jpeg) | 远程视频 [video-002](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-2757650c-5b9e-4cbf-a3f5-13d8b6a1cd82.mp4) |
| `video` reference-image video | 参考这两个验证图标，做一个简短干净的产品揭示动画 | 远程图片 [image-002](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-128f4cbe-67a5-4e5a-8014-021a827781c7.jpeg)、[image-003](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-153c638c-3599-4cbe-9763-8c3a54bb4e3b.jpeg) | 远程视频 [video-003](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-8ca95e5e-f9b8-4410-aa7f-27d955fe1921.mp4) |
| `video-edit` | 给终端窗口加一点淡蓝色的验证光晕 | 远程视频 [video-001](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-e16c4b37-d3a7-4dc0-932f-c62610ed5b3e.mp4) | 远程视频 [video-edit-001](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-15ed916c-38d0-4929-88ab-2c8d70fc9fc8.mp4) |
| `video-extend` | 保持同样的简单挥手动作，再持续一小会儿 | 远程视频 [video-001](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-e16c4b37-d3a7-4dc0-932f-c62610ed5b3e.mp4) | 远程视频 [video-extend-001](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-b6f66d9e-a0b5-4c07-9c7f-aa47e1f67f8d.mp4) |

说明：
- `video-extend` 使用的输入视频是前一条 `video` text-to-video 生成的 [video-001](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-e16c4b37-d3a7-4dc0-932f-c62610ed5b3e.mp4)。
- 本次验证里，源视频总时长为 `2s`，扩展后总时长为 `4s`。
- 这张表中的远程 URL 都保留为可点击链接，便于手工复核；涉及本地文件的引用已经归档到 `docs/project/tests/2026-05-21T12-20-32+0800/`。

### 本轮真实测试参数覆盖盘点

已实际使用过的参数：

- `tts`：`--json`、`--text` 或位置文本、`--output`
- `stt`：`--json`、`--file`
- `image`：`--json`、`--prompt` 或位置提示词、`--count`、`--response-format url`、`--output-dir`
- `image-edit`：`--json`、`--image`、`--prompt`
- `video`：`--json`、`--prompt`、`--duration`、`--image-url`、`--reference-image-url`
- `video-edit`：`--json`、`--video-url`、`--prompt`
- `video-extend`：`--json`、`--video-url`、`--prompt`、`--duration`
- `usage`：`--json`

本轮未实际使用、建议进入下一轮真实测试的参数：

- `image`：`--aspect-ratio`、`--resolution`、`--model`、`--output-file`、`--timeout`
- `image-edit`：`--aspect-ratio`、`--resolution`、`--response-format b64_json`、`--output-file`、`--model`、`--timeout`、远程 URL 输入分支
- `video`：`--image`、`--reference-image`、`--aspect-ratio`、`--resolution`、`--model`、`--timeout`
- `video-edit`：`--video`、`--model`、`--timeout`
- `video-extend`：`--video`、`--model`、`--timeout`
- `tts`：`--list-voices`、`--voice-id`、`--language`、`--output-format`、`--sample-rate`、`--bit-rate`、`--optimize-streaming-latency`、`--text-normalization`、`--model`、`--timeout`
- `stt`：`--url`、`--language`、`--format false`、`--audio-format`、`--sample-rate`、`--multichannel`、`--channels`、`--diarize`、`--keyterm`、`--filler-words`、`--model`、`--timeout`
- `stt-stream`：整条真实链路尚未做，相关参数也都未做真实验证

### 下一轮真实测试建议

说明：下面这组建议已在后续“阶段 2b：真实参数补测”中按优先级执行，保留在这里用于追溯参数补测来源；当前仍未覆盖项以阶段 2b 的“仍未覆盖参数”为准。

建议按下面顺序补，优先覆盖真实价值高、且能顺带验证本地 path 支持的分支：

1. `image --aspect-ratio 1:1 --resolution 1k --output-file ...`
2. `image-edit --image <远程URL> --response-format b64_json --output-file ...`
3. `video --image <本地路径> --aspect-ratio 9:16 --resolution 720p`
4. `video --reference-image <本地路径>` 多参考图分支
5. `video-edit --video <本地路径>`
6. 原计划测试 `video-extend --video <本地路径> --duration 2`；阶段 2b 后已确认该能力不进入 CLI 能力面，实际使用 `--video-url`
7. `tts --list-voices --json`
8. `tts` 选一个非默认 `--voice-id`，同时带 `--language`、`--output-format`
9. `stt --url <远程音频>`，补远程 URL 分支
10. `stt` 高级参数组合：`--diarize --keyterm ... --filler-words`

这一轮先不要求把所有参数都跑满，只优先覆盖“本地 path 新增支持”和“高频用户会直接遇到的可见参数”。

## 2026-05-21 阶段 2b：真实参数补测

状态：部分通过。图片、音频和大部分本地 path 视频参数已通过；`video-extend --video` 两个本地 MP4 样本均进入上游服务端后生成失败。基于该结果，`video-extend` 的本地 path 能力已移除，只保留 `--video-url`。

### 测试环境

- 执行二进制：`target/debug/grok-cli`，因为全局安装的 `grok-cli 0.1.0` 尚未包含新本地 path 参数。
- OAuth 状态：已登录，测试期间没有触发 `auth_relogin_required`。
- 临时输出目录：`.tmp/parameter-validation/2026-05-21T12-26-07+0800/`，不会进入 git。
- 文档归档样本目录：`docs/project/tests/2026-05-21T12-26-07+0800/`。
- 执行计划与逐项结果：[parameter-validation-plan.md](./parameter-validation-plan.md)。

### 参数补测结果

| 编号 | 能力 | 本轮覆盖参数 | 中文提示词 / 文本 | 真实引用输入 | 真实输出 / 结果 |
| --- | --- | --- | --- | --- | --- |
| P0-1 | `video` image-to-video | `--image`、`--aspect-ratio 9:16`、`--resolution 720p`、`--timeout` | 让这个本地验证吉祥物轻轻挥手，画面竖屏构图 | 本地图片 [image-001.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/image-001.png) | 通过，远程视频 [p0-1-video](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-b253d9b1-a64d-4799-be6f-127a3757c981.mp4)，`modality=image` |
| P0-2 | `video` reference-image video | `--reference-image` 多值、`--timeout` | 参考这两个本地图标，做一个简短的验证徽章揭示动画 | 本地图片 [image-001.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/image-001.png)、[image-002.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/image-002.png) | 通过，远程视频 [p0-2-video](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-7329c966-7850-47e8-a012-261b954c2121.mp4) |
| P0-3 | `video-edit` | `--video`、`--timeout` | 给这个本地视频里的终端窗口加一点蓝色验证光晕 | 本地视频 [source-video-001.mp4](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/videos/source-video-001.mp4) | 通过，远程视频 [p0-3-video-edit](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-3fbb7daf-eac3-4d22-9271-50d1ba1ecba8.mp4)，`modality=edit` |
| P0-4 | `video-extend` | 原计划验证 `--video`、`--duration 2`、`--timeout` | 保持同样的动作，再自然延续两秒 | 本地视频 [source-video-001.mp4](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/videos/source-video-001.mp4) | 未通过，上游终态返回 `request_failed` / internal error；已移除 `--video` 能力 |
| P0-4b | `video-extend` | 原计划验证 `--video`、`--duration 2`、`--timeout` | 保持同样的画面风格，再自然延续两秒 | 本地视频 [source-video-image-001.mp4](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/videos/source-video-image-001.mp4) | 未通过，上游终态返回 `request_failed` / internal error；已移除 `--video` 能力 |
| P1-1 | `image` | `--aspect-ratio 1:1`、`--resolution 1k`、`--output-file`、`--timeout` | 一个 1:1 的极简 Grok CLI 验证徽章，清晰边缘 | 无 | 通过，本地图片 [p1-1-image-output.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/p1-1-image-output.png) |
| P1-2 | `image-edit` | 远程 URL 输入、`--response-format b64_json`、`--output-file`、`--timeout` | 把牌子文字改得更像命令行验证通过提示 | 远程图片 [image-001](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-ddebf8f7-1ffc-44b4-82bb-a4bc209a520c.jpeg) | 通过，本地图片 [p1-2-image-edit-output.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/p1-2-image-edit-output.png) |
| P1-3 | `tts` | `--list-voices` | 无 | 无 | 通过，返回 71 个 voice；抽样包含 `ara`、`eve`、`leo`、`rex`、`sal` |
| P1-4 | `tts` | `--voice-id ara`、`--language en`、`--output-format mp3`、`--sample-rate 24000`、`--timeout` | `Grok CLI parameter validation sample.` | 无 | 通过，本地音频 [p1-4-tts-ara.mp3](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/audio/p1-4-tts-ara.mp3)，`sample_rate=24000` |
| P1-5 | `stt` | `--language en`、`--format false`、`--diarize`、多 `--keyterm`、`--filler-words`、`--timeout` | 无 | 本地音频 [p1-4-tts-ara.mp3](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/audio/p1-4-tts-ara.mp3) | 通过，转写文本 `Grok CLI parameter validation sample.`，`duration=2.71` |

### 仍未覆盖参数

- `--model`：需要确认可用模型别名后再测，避免传错模型造成无意义失败。
- `tts --bit-rate`、`--optimize-streaming-latency`、`--text-normalization`：低频透传参数，本轮未测。
- `stt --url`：需要稳定公开音频 URL；本轮不临时上传私有音频。
- `stt --audio-format`、`--sample-rate`：需要原始 PCM 或无容器元数据样本。
- `stt --multichannel`、`--channels`：需要真实多声道音频。
- `stt-stream`：用户已确认暂不做更深层 WebSocket mock / 分块发送测试。

### 观察

- 本地图片 path 在 `video --image` 和 `video --reference-image` 中均已真实成功。
- 本地视频 path 在 `video-edit --video` 中已真实成功。
- `video-extend --video` 的失败发生在上游生成终态；两次请求均已进入上游并获得 request id，不是 CLI 本地 path 编码、文件读取或认证失败。当前 CLI 不再提供该参数，用户需要使用可公开访问的 `--video-url`。
- 本批次归档样本目录大小约 `1.0M`。

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
