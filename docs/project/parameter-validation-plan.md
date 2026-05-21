# Parameter Validation Plan

这份文档用于补齐发布前真实媒体验证里尚未覆盖的 CLI 参数。执行顺序按风险和用户价值排序，先验证新增的本地 path 支持，再补高频输出、格式和高级音频参数。

## 目标

- 真实验证未覆盖的关键参数是否能被上游接受。
- 重点确认本地 path 输入会被 CLI 正确转换并发送。
- 每条真实测试都记录输入资产、提示词、参数、生成结果和后续动作。
- 原始输出继续放在 `.tmp/`，可复核的本地样本放在 `docs/project/tests/{测试时间戳}/`。

## 测试批次

- 时间戳：`2026-05-21T12-26-07+0800`
- 归档目录：`docs/project/tests/2026-05-21T12-26-07+0800/`
- 临时输出目录：`.tmp/parameter-validation/2026-05-21T12-26-07+0800/`
- 执行二进制：`target/debug/grok-cli`
- OAuth 状态：复用本机 `~/.grok-cli/auth.json`

## P0 本地 Path 视频输入

状态：部分通过。`video --image`、`video --reference-image`、`video-edit --video` 已真实通过；`video-extend --video` 两个本地 MP4 样本都进入上游服务端，但生成端返回 internal error。基于该结果，`video-extend` 的本地 path 能力已从 CLI 能力面移除，只保留 `--video-url`。

| 编号 | 能力 | 本轮新增覆盖参数 | 输入资产 | 中文提示词 | 验收 |
| --- | --- | --- | --- | --- | --- |
| P0-1 | `video` image-to-video | `--image`、`--aspect-ratio`、`--resolution`、`--timeout` | `images/image-001.png` | 让这个本地验证吉祥物轻轻挥手，画面竖屏构图 | 返回远程视频 URL，`modality=image` |
| P0-2 | `video` reference-image video | `--reference-image` 多值 | `images/image-001.png`、`images/image-002.png` | 参考这两个本地图标，做一个简短的验证徽章揭示动画 | 返回远程视频 URL |
| P0-3 | `video-edit` | `--video`、`--timeout` | `source-video-001.mp4` | 给这个本地视频里的终端窗口加一点蓝色验证光晕 | 返回远程视频 URL，`modality=edit` |
| P0-4 | `video-extend` | 原计划验证 `--video`、`--duration`、`--timeout` | `source-video-001.mp4` | 保持同样的动作，再自然延续两秒 | 未通过后移除 `--video` 能力，只保留 `--video-url` |

### P0 执行结果

| 编号 | 结果 | 真实输入 | 真实输出 / 错误 |
| --- | --- | --- | --- |
| P0-1 | 通过 | 本地图片 [image-001.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/image-001.png) | 远程视频 [p0-1-video](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-b253d9b1-a64d-4799-be6f-127a3757c981.mp4)，`modality=image`，`aspect_ratio=9:16`，`duration=8` |
| P0-2 | 通过 | 本地图片 [image-001.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/image-001.png)、[image-002.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/image-002.png) | 远程视频 [p0-2-video](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-7329c966-7850-47e8-a012-261b954c2121.mp4)，`modality=image`，`duration=8` |
| P0-3 | 通过 | 本地视频 [source-video-001.mp4](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/videos/source-video-001.mp4) | 远程视频 [p0-3-video-edit](https://vidgen.x.ai/xai-vidgen-bucket/xai-video-3fbb7daf-eac3-4d22-9271-50d1ba1ecba8.mp4)，`modality=edit`，`duration=2` |
| P0-4 | 未通过 | 本地视频 [source-video-001.mp4](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/videos/source-video-001.mp4) | 上游返回 `request_failed`：`Video generation failed due to an internal error. Please try again.` |
| P0-4b | 未通过 | 本地视频 [source-video-image-001.mp4](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/videos/source-video-image-001.mp4) | 上游返回 `request_failed`：`Video generation failed due to an internal error. Please try again.` |

P0-4 观察：
- 两次 `video-extend --video` 都已经拿到上游 request id，说明本地 path 已经被 CLI 转成 `data:video/mp4;base64,...` 并被服务端接收。
- 失败发生在视频生成终态，不是本地文件读取、参数解析或鉴权失败。
- 基于当前真实结果，`video-extend` 不再支持本地视频 path。用户需要先把本地视频上传到可公开访问的 URL，再使用 `--video-url`。

## P1 高频图片与音频参数

状态：通过。

| 编号 | 能力 | 本轮新增覆盖参数 | 输入资产 | 中文提示词 / 文本 | 验收 |
| --- | --- | --- | --- | --- | --- |
| P1-1 | `image` | `--aspect-ratio`、`--resolution`、`--output-file`、`--timeout` | 无 | 一个 1:1 的极简 Grok CLI 验证徽章，清晰边缘 | 写入本地图片文件 |
| P1-2 | `image-edit` | 远程 URL 输入、`--response-format b64_json`、`--output-file` | 上轮远程图片 `image-001` | 把牌子文字改得更像命令行验证通过提示 | 写入本地图片文件 |
| P1-3 | `tts` | `--list-voices` | 无 | 无 | 返回 voices 列表 |
| P1-4 | `tts` | `--voice-id`、`--language`、`--output-format`、`--sample-rate`、`--timeout` | 无 | Grok CLI parameter validation sample. | 写入本地音频文件 |
| P1-5 | `stt` | `--diarize`、`--keyterm`、`--filler-words`、`--timeout` | P1-4 生成音频 | 无 | 返回 transcript 和结构化字段 |

### P1 执行结果

| 编号 | 结果 | 真实输入 | 真实输出 / 观察 |
| --- | --- | --- | --- |
| P1-1 | 通过 | 无 | 本地图片 [p1-1-image-output.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/p1-1-image-output.png)，`aspect_ratio=1:1` |
| P1-2 | 通过 | 远程图片 [image-001](https://imgen.x.ai/xai-imgen/xai-tmp-imgen-ddebf8f7-1ffc-44b4-82bb-a4bc209a520c.jpeg) | 本地图片 [p1-2-image-edit-output.png](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/images/p1-2-image-edit-output.png) |
| P1-3 | 通过 | 无 | `tts --list-voices` 返回 71 个 voice；抽样包括 `ara`、`eve`、`leo`、`rex`、`sal` |
| P1-4 | 通过 | 文本 `Grok CLI parameter validation sample.` | 本地音频 [p1-4-tts-ara.mp3](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/audio/p1-4-tts-ara.mp3)，`codec=mp3`，`sample_rate=24000` |
| P1-5 | 通过 | 本地音频 [p1-4-tts-ara.mp3](/Users/seanmo/ai/develop/Grok/grok-cli/docs/project/tests/2026-05-21T12-26-07+0800/audio/p1-4-tts-ara.mp3) | 转写文本 `Grok CLI parameter validation sample.`，`language=English`，`duration=2.71` |

## 暂缓项

- `stt --url`：需要稳定、公开可访问的远程音频 URL。当前先不临时上传私有音频；本轮只补了本地文件高级参数。
- `stt --audio-format` / `--sample-rate`：适合用原始 PCM 样本验证，当前已有容器格式音频不强行覆盖。
- `stt --multichannel` / `--channels`：需要真实多声道样本。
- `stt-stream`：用户已确认暂不做更深层 WebSocket mock / 分块发送测试，本轮不扩展真实流式覆盖。
- `tts --bit-rate`、`--optimize-streaming-latency`、`--text-normalization`：本轮先覆盖 voice、language、format、sample rate；这些低频透传参数后续可单独补。
- `--model`：当前默认模型已通过真实请求覆盖；模型覆盖要避免传入未知模型导致无意义失败，后续有确定可用模型别名时再补。

## 记录格式

每条测试完成后记录：

- 使用命令和新增覆盖参数。
- 输入资产路径或远程 URL。
- 输出文件路径或远程 URL。
- JSON 主字段是否存在。
- 失败时记录错误码和是否需要重试。

## 验收标准

- P0 四条本地 path 视频输入至少各成功一次：部分达成，3 条通过；`video-extend --video` 两个样本均为上游生成失败，能力已从 CLI 移除。
- P1 图片和音频高频参数至少成功覆盖一次：已达成。
- 结果回写到 `pre-release-validation-results.md`：已达成。
- 可复核本地样本归档到本批次 `docs/project/tests/2026-05-21T12-26-07+0800/`：已达成。
