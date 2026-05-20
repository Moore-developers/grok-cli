# SuperGrok Media Capability Completion Plan

这份文档把 `grok-cli` 的 `image` / `tts` / `stt` 能力差异整理成可执行任务计划。目标是对齐 Hermes Agent 已经验证过的 xAI 调用形态，并继续补齐 xAI 官方文档中 SuperGrok 媒体能力暴露出来的参数、返回结构和后续扩展入口。

## 来源

本计划基于以下来源整理：

- Hermes Agent xAI image provider: `/Users/seanmo/AI/packages/hermes-agent/plugins/image_gen/xai/__init__.py`
- Hermes Agent TTS: `/Users/seanmo/AI/packages/hermes-agent/tools/tts_tool.py`
- Hermes Agent STT: `/Users/seanmo/AI/packages/hermes-agent/tools/transcription_tools.py`
- 当前 CLI image 实现: [`src/task/image.rs`](../../src/task/image.rs)
- 当前 CLI audio 实现: [`src/task/audio.rs`](../../src/task/audio.rs)
- 当前 CLI 参数定义: [`src/args.rs`](../../src/args.rs)
- xAI Imagine docs: <https://docs.x.ai/developers/model-capabilities/imagine>
- xAI TTS docs: <https://docs.x.ai/developers/model-capabilities/audio/text-to-speech>
- xAI STT docs: <https://docs.x.ai/developers/model-capabilities/audio/speech-to-text>

## 测试门槛

每补一个能力或参数，都必须同时补测试。未补测试的能力不能标记为完成。

最低测试要求：

- 模块级测试：覆盖参数校验、请求体构造、multipart form 构造、响应解析。
- 命令级 stub 测试：覆盖 CLI 参数到上游请求的真实串联，优先使用现有 `tests/task_audio_commands.rs` 和 `tests/task_image_gen_commands.rs`。
- 文档同步：用户可见参数变更必须同步对应 `docs/commands/*.md`。
- 回归执行：单项开发时至少跑对应模块或命令测试；阶段完成前跑 `cargo test --quiet`。

推荐测试分层：

| 改动类型 | 必测位置 |
|---|---|
| 新 CLI 参数 | `src/task/*` 模块级构造测试 + `tests/task_*_commands.rs` 命令级参数测试 |
| 新返回字段 | 响应解析单元测试 + JSON 输出命令级 stub 测试 |
| 新校验规则 | 模块级 `validate_*` 测试 + 命令级失败输出测试 |
| 新上游执行形态 | 请求构造测试 + stub server / mocked transport 测试 |
| 新命令或新模式 | `--help` / contract 回归测试 + 命令级成功和失败路径测试 |

## 当前缺口总览

| 能力 | 当前状态 | 主要缺口 | 优先级 |
|---|---|---|---|
| `stt` batch transcription | 只支持本地 `file + language + format=true` | `url`、`audio_format`、`sample_rate`、`multichannel`、`channels`、`diarize`、`keyterm`、`filler_words`、结构化返回 | P0 |
| `tts` synthesis | 支持 `text`、`voice_id`、`language`，仅 `.wav` 时隐式发送固定 `output_format` | 显式 output format、sample rate、bit rate、stream latency、text normalization、voice discovery | P0 |
| `image` generation | 支持 `prompt`、`model`、`aspect_ratio`、`resolution`、单图输出 | `count`、显式 `response_format`、多图返回、批量落盘策略 | P1 |
| `stt` streaming | 无 | WebSocket 实时转写 | P2 |
| `image` editing | 无 | image edit 和 multi-image edit | P2 |
| Imagine video follow-up | 已有 generation / image-to-video / reference image video 主路径 | video editing / extension 待单独确认接口文档 | P3 |

## Phase 16.1: STT Batch Completion

目标：先补齐官方 STT batch 接口的参数面和结构化输出。STT 当前缺口最大，而且大多数改动都能通过现有 multipart 上游执行层完成。

### STT-1. 参数与校验

- [x] 在 `SttOptions` 增加 `--url <URL>`，与位置 `PATH` / `--file` 互斥。
- [x] 在 `SttOptions` 增加 `--format <true|false>`，默认保持 `true`，避免破坏现有行为。
- [x] 增加 `--audio-format <FORMAT>`。
- [x] 增加 `--sample-rate <HZ>`。
- [x] 增加 `--multichannel`。
- [x] 增加 `--channels <CHANNELS>`，先按逗号分隔字符串透传，例如 `0,1`。
- [x] 增加 `--diarize`。
- [x] 增加可重复 `--keyterm <TERM>`。
- [x] 增加 `--filler-words`。

测试要求：

- [x] 模块级测试覆盖 `file`、`url` 二选一校验。
- [x] 模块级测试覆盖 `file + url` 冲突。
- [x] 模块级测试覆盖缺少输入时返回 `invalid_args`。
- [x] 模块级测试覆盖所有新增字段进入 multipart form。
- [x] 命令级 stub 测试验证新增参数会出现在发往 `/v1/stt` 的请求中。

### STT-2. URL 转写

- [x] `build_stt_form` 支持只发送 `url` 而不读取本地文件。
- [x] 保持本地文件路径的现有行为不变。
- [x] 错误信息区分“缺少输入”和“文件不存在”。

测试要求：

- [x] 模块级测试覆盖 URL 模式不访问本地文件。
- [x] 命令级 stub 测试覆盖 `grok-cli stt --json --url https://...`。
- [x] 命令级失败测试覆盖 `--url` 和 `--file` 同时传入。

### STT-3. 结构化响应输出

- [x] `SttData` 保留 `transcript`，继续兼容现有 SKILL 消费。
- [x] 增加可选 `language`。
- [x] 增加可选 `duration`。
- [x] 增加可选 `words`。
- [x] 增加可选 `channels`。
- [x] 非 JSON 输出仍优先显示 `transcript`，只附加简短元信息。

测试要求：

- [x] 模块级测试覆盖只返回 `text` 的旧响应。
- [x] 模块级测试覆盖含 `language` / `duration` / `words` / `channels` 的新响应。
- [x] 命令级 stub 测试确认 `--json` 输出包含新增字段。
- [x] contract 回归确认 `data.transcript` 仍存在。

### STT-4. 文档同步

- [x] 更新 [`docs/commands/stt.md`](../commands/stt.md)。
- [x] 更新 [`docs/project/acceptance.md`](./acceptance.md) 的媒体验收样例。
- [ ] 需要时更新 [`docs/reference/samples.md`](../reference/samples.md) 的 JSON 样例。

完成标准：

- [x] `cargo test --quiet task_audio_commands` 通过。
- [x] `cargo test --quiet` 通过。
- [x] STT 新增参数和结构化输出都有文档说明。

## Phase 16.2: TTS Parameter Completion

目标：补齐官方 TTS 参数面，同时保持当前“生成音频并写入本地文件”的主路径简单稳定。

### TTS-1. 显式 output format

- [ ] 在 `TtsOptions` 增加 `--output-format <FORMAT>`，建议枚举先支持 `mp3`、`wav`，后续按官方能力扩展。
- [ ] 增加 `--sample-rate <HZ>`。
- [ ] 增加 `--bit-rate <BPS>`。
- [ ] 明确 `--output` 扩展名与 `--output-format` 不一致时的行为，建议先返回 `invalid_args`。
- [ ] 默认行为保持 `mp3`。

测试要求：

- [ ] 模块级测试覆盖默认不发送冗余 `output_format` 或保持现有最小 payload。
- [ ] 模块级测试覆盖显式 `mp3` / `wav` output format。
- [ ] 模块级测试覆盖 sample rate 和 bit rate 进入 payload。
- [ ] 模块级测试覆盖输出扩展名与显式格式冲突。
- [ ] 命令级 stub 测试验证请求 JSON 包含 `output_format`。

### TTS-2. 官方高级参数

- [ ] 增加 `--optimize-streaming-latency <MODE>` 并按官方字段透传。
- [ ] 增加 `--text-normalization <MODE>` 并按官方字段透传。
- [ ] 允许 `--language auto`。
- [ ] 继续允许自定义 `--voice-id`。

测试要求：

- [ ] 模块级测试覆盖两个高级参数进入 payload。
- [ ] 模块级测试覆盖 `language=auto`。
- [ ] 命令级 stub 测试覆盖完整高级参数组合。

### TTS-3. Voice discovery

由于现有 `grok-cli tts "..."` 使用位置文本，`grok-cli tts voices` 会和普通文本合成有歧义。第一版采用模式参数：

- [ ] 增加 `grok-cli tts --list-voices`。
- [ ] 调用 `GET /v1/tts/voices`。
- [ ] `--json` 输出 voice 列表原始结构或稳定包装结构。
- [ ] 人类输出显示 voice id、名称、类型等关键字段。

测试要求：

- [ ] 命令级 stub 测试覆盖 `tts --list-voices --json`。
- [ ] 响应解析测试覆盖空列表和多个 voice。
- [ ] help / contract 测试确认新参数可见。

### TTS-4. 文档同步

- [ ] 更新 [`docs/commands/tts.md`](../commands/tts.md)。
- [ ] 更新 [`docs/project/acceptance.md`](./acceptance.md)。
- [ ] 需要时更新 [`docs/reference/samples.md`](../reference/samples.md)。

完成标准：

- [ ] `cargo test --quiet task_audio_commands` 通过。
- [ ] `cargo test --quiet` 通过。
- [ ] 默认 `grok-cli tts "hello"` 行为不变。

## Phase 16.3: Image Generation Completion

目标：补齐 Imagine image generation 的多图和响应格式控制，同时保留当前 `image` 主字段，避免破坏已有集成。

### IMG-1. Count 与 response format

- [ ] 增加 `--count <N>`，默认 `1`，校验范围 `1..=10`。
- [ ] 增加 `--response-format <url|b64_json>`。
- [ ] `--output-file` 继续隐式要求 `b64_json`。
- [ ] 当 `--output-file` 与 `--count > 1` 同时出现时返回 `invalid_args`。
- [ ] 评估并实现 `--output-dir <PATH>`，用于多图 b64 落盘。

测试要求：

- [ ] 模块级测试覆盖 `count` 和 `response_format` 进入请求体。
- [ ] 模块级测试覆盖 `count=0` / `count=11` 校验失败。
- [ ] 模块级测试覆盖 `--output-file + --count > 1` 失败。
- [ ] 命令级 stub 测试验证多图请求。

### IMG-2. 多图响应

- [ ] `ImageGenData` 保留 `image`，继续表示第一张图。
- [ ] 增加 `images: Vec<String>`，用于返回完整图片列表。
- [ ] `--json` 输出 `image` 和 `images`。
- [ ] 非 JSON 输出显示第一张图和总数；必要时逐行列出全部图片。
- [ ] b64 多图落盘时返回本地路径列表。

测试要求：

- [ ] 模块级测试覆盖多个 `url`。
- [ ] 模块级测试覆盖多个 `b64_json`。
- [ ] 命令级 stub 测试确认 JSON 输出包含 `images`。
- [ ] contract 回归确认 `data.image` 仍存在。

### IMG-3. 文档同步

- [ ] 更新 [`docs/commands/image.md`](../commands/image.md)。
- [ ] 更新 [`docs/project/acceptance.md`](./acceptance.md)。
- [ ] 需要时更新 [`docs/reference/samples.md`](../reference/samples.md)。

完成标准：

- [ ] `cargo test --quiet task_image_gen_commands` 通过。
- [ ] `cargo test --quiet` 通过。
- [ ] 单图旧用法仍返回 `data.image`。

## Phase 16.4: Streaming STT

目标：新增实时语音转写入口。这个能力协议形态不同，不和 batch `stt` 混在一个实现里。

建议 CLI：

```bash
grok-cli stt-stream --file ./sample.wav --language en --interim-results
```

任务：

- [ ] 确认 WebSocket 依赖选择和当前 `Cargo.toml` 影响。
- [ ] 新增 `stt-stream` 顶层命令或隐藏实验入口。
- [ ] 支持官方 WebSocket 参数：`interim_results`、`endpointing`、`encoding`、`sample_rate`、`language`、`diarize`、`filler_words`、`multichannel`、`channels`、`keyterm`。
- [ ] 定义 JSON event 输出格式。
- [ ] 定义人类输出格式，区分 interim 和 final。

测试要求：

- [ ] 单元测试覆盖 WebSocket URL / query / init payload 构造。
- [ ] 单元测试覆盖 event parser。
- [ ] 命令级测试覆盖参数校验。
- [ ] 如果引入 mock WebSocket server，补成功事件流测试。

完成标准：

- [ ] 先以实验入口落地，测试覆盖协议构造和事件解析。
- [ ] 文档明确这是 streaming STT，不替代 batch `stt`。

## Phase 16.5: Image Editing

目标：把 Imagine image edit / multi-image edit 做成独立能力，不污染 `image` 生成主命令。

建议 CLI：

```bash
grok-cli image-edit --image ./source.png --prompt "Make it cinematic"
grok-cli image-edit --image ./a.png --image ./b.png --image ./c.png --prompt "Blend these references"
```

任务：

- [ ] 再次核对官方 image edit endpoint 和 multipart / JSON payload 形态。
- [ ] 新增 `image-edit` 顶层命令。
- [ ] 支持最多 3 张输入图。
- [ ] 支持本地路径和 URL，必要时分两期实现。
- [ ] 复用 `image` 的输出解析和落盘策略。

测试要求：

- [ ] 模块级测试覆盖 1 张和 3 张输入图。
- [ ] 模块级测试覆盖超过 3 张失败。
- [ ] 命令级 stub 测试覆盖成功请求。
- [ ] contract 回归确认新命令出现在 help。

完成标准：

- [ ] `image` 旧命令行为不变。
- [ ] `image-edit` 有独立文档和测试。

## Phase 16.6: Imagine Video Follow-up

现有 `video` 已覆盖 text-to-video、image-to-video、reference image video 主路径。Imagine overview 还提到 video editing 和 video extension，这两项先作为后续确认任务。

任务：

- [ ] 核对官方 video editing / extension endpoint。
- [ ] 对比当前 [`src/task/video.rs`](../../src/task/video.rs) 参数面。
- [ ] 决定是扩展 `video` 还是新增 `video-edit` / `video-extend`。
- [ ] 每个新增能力按相同测试门槛补测试。

完成标准：

- [ ] 形成单独 video follow-up 计划或直接拆进下一阶段任务。

## 执行顺序

推荐顺序：

1. Phase 16.1 STT batch completion。
2. Phase 16.2 TTS parameter completion。
3. Phase 16.3 Image generation completion。
4. Phase 16.4 Streaming STT。
5. Phase 16.5 Image editing。
6. Phase 16.6 Imagine video follow-up。

每个 phase 完成时都必须满足：

- [ ] 代码实现完成。
- [ ] 模块级测试完成。
- [ ] 命令级 stub 测试完成。
- [ ] 用户文档更新完成。
- [ ] 对应 targeted tests 通过。
- [ ] `cargo test --quiet` 通过。
