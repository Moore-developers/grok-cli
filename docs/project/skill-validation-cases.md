# Skill Validation Cases

这份文档用于验证 `skills/grok-cli` 在 Codex 和 Claude Code 中是否能覆盖 `grok-cli` 的全部公开能力。验证重点不是上游回答质量，而是：

- skill 是否能选对命令
- 参数是否成形
- 本地文件是否会被正确带入
- 不支持的能力是否会被明确拦住

## 使用方式

在 Codex 或 Claude Code 中：

1. 安装或加载 `skills/grok-cli`
2. 直接输入下面的自然语言请求
3. 检查代理是否选择了预期的 `grok-cli` 命令
4. 对需要本地文件的用例，直接在提示词里给出路径，或明确说“处理这个文件”

## 全能力验证矩阵

| 编号 | 能力 | 用户在 Codex / Claude Code 中可直接说的话 | 预期 skill 路由 |
| --- | --- | --- | --- |
| A1 | `login` | 帮我登录 Grok | `grok-cli login` |
| A2 | `status` | 看一下我现在的 Grok 登录状态 | `grok-cli status --json` |
| A3 | `refresh` | 刷新一下 Grok 的登录状态 | `grok-cli refresh --json` |
| A4 | `logout` | 把本地 Grok 登录退出掉 | `grok-cli logout --json` |
| A5 | `state` | 读取一下本地认证状态摘要 | `grok-cli state --json` |
| A6 | `model` | 看一下现在默认文本模型是什么 | `grok-cli model --json` |
| A7 | `usage` | 看一下本地 usage 统计 | `grok-cli usage --json` |
| A8 | `chat` | 用 Grok 总结一下最近关于 Rust CLI 设计的讨论，返回结构化结果 | `grok-cli chat --json --prompt "..."` |
| A9 | `search` | 搜索一下 X 上大家今天怎么评价 Grok CLI | `grok-cli search --json --query "..."` |
| A10 | `image` | 生成一张 1:1 的极简终端图标，保存到本地 | `grok-cli image --json --prompt "..." --aspect-ratio 1:1 --output-file ...` |
| A11 | `image` 多图 | 生成 3 张不同风格的终端吉祥物并保存到目录 | `grok-cli image --json --prompt "..." --count 3 --output-dir ...` |
| A12 | `image-edit` 本地图 | 把这张图片改得更像命令行工具封面：`./source.png` | `grok-cli image-edit --json --image ./source.png --prompt "..."` |
| A13 | `image-edit` 多图 | 用这两张图融合成一个验证徽章：`./a.png ./b.png` | `grok-cli image-edit --json --image ./a.png --image ./b.png --prompt "..."` |
| A14 | `image-edit` 远程图 | 编辑这个远程图片并保存到本地：`https://...` | `grok-cli image-edit --json --image https://... --output-file ... --prompt "..."` |
| A15 | `video` text-to-video | 生成一个 8 秒的终端风格短视频 | `grok-cli video --json --prompt "..." --duration 8` |
| A16 | `video` 本地图转视频 | 用这张本地图片做一个短视频：`./source.png` | `grok-cli video --json --prompt "..." --image ./source.png` |
| A17 | `video` 远程图转视频 | 用这个远程图片做一个短视频：`https://...` | `grok-cli video --json --prompt "..." --image-url https://...` |
| A18 | `video` 本地参考图 | 用这两个本地参考图做一个产品揭示视频：`./a.png ./b.png` | `grok-cli video --json --prompt "..." --reference-image ./a.png --reference-image ./b.png` |
| A19 | `video` 远程参考图 | 参考这两个远程图生成一个短视频：`https://... https://...` | `grok-cli video --json --prompt "..." --reference-image-url https://... --reference-image-url https://...` |
| A20 | `video-edit` 本地视频 | 编辑这个本地视频，让画面更有电影感：`./source.mp4` | `grok-cli video-edit --json --video ./source.mp4 --prompt "..."` |
| A21 | `video-edit` 远程视频 | 编辑这个远程视频，让颜色更冷一点：`https://...` | `grok-cli video-edit --json --video-url https://... --prompt "..."` |
| A22 | `video-extend` 远程视频 | 把这个视频再延长两秒：`https://example.com/source.mp4` | `grok-cli video-extend --json --video-url https://example.com/source.mp4 --duration 2 --prompt "..."` |
| A23 | `tts` 列表 | 列出当前可用的 TTS 声音 | `grok-cli tts --json --list-voices` |
| A24 | `tts` 合成 | 把这段文字转成 ara 声音的 mp3 并保存 | `grok-cli tts --json --text "..." --voice-id ara --output ... --output-format mp3` |
| A25 | `stt` 本地音频 | 转写这个音频，并保留关键词 Grok 和 CLI：`./sample.wav` | `grok-cli stt --json --file ./sample.wav --keyterm Grok --keyterm CLI` |
| A26 | `stt` 远程音频 | 转写这个远程音频：`https://example.com/sample.wav` | `grok-cli stt --json --url https://example.com/sample.wav` |
| A27 | `stt-stream` | 用实时方式转写这个音频：`./sample.wav` | `grok-cli stt-stream --json --file ./sample.wav ...` |

## 本地文件场景

这些是 skill 必须正确处理的本地文件输入方式。用户不需要懂 CLI 参数名，只需要把文件路径告诉 skill：

| 输入类型 | 用户说法示例 | 预期命令 |
| --- | --- | --- |
| 本地单图编辑 | 处理这张图：`./source.png` | `image-edit --image ./source.png` |
| 本地多图编辑 | 处理这两张图：`./a.png ./b.png` | `image-edit --image ./a.png --image ./b.png` |
| 本地图转视频 | 用这张图做视频：`./source.png` | `video --image ./source.png` |
| 本地参考图视频 | 用这两张图当参考：`./a.png ./b.png` | `video --reference-image ./a.png --reference-image ./b.png` |
| 本地视频编辑 | 处理这个视频：`./source.mp4` | `video-edit --video ./source.mp4` |
| 本地音频转写 | 转写这个音频：`./sample.wav` | `stt --file ./sample.wav` |
| 本地音频流式转写 | 实时转写这个音频：`./sample.wav` | `stt-stream --file ./sample.wav` |

## 不支持与负向用例

### N1. 不应编造本地视频扩展命令

用户提示词：

```text
把这个本地视频延长两秒：./source.mp4
```

预期行为：

- 不应生成 `grok-cli video-extend --video ./source.mp4`
- 应明确说明 `video-extend` 当前只支持 `--video-url`
- 应引导先把本地视频上传到可公开访问的 URL，再继续扩展

### N2. 不应把图片编辑误路由到图片生成

用户提示词：

```text
用这张图做一点改动：./source.png
```

预期行为：

- 应选择 `image-edit`
- 不应选择 `image`

### N3. 不应把视频编辑误路由到视频生成

用户提示词：

```text
修改这个已有视频的颜色：./source.mp4
```

预期行为：

- 应选择 `video-edit`
- 不应选择 `video`

### N4. 不应把普通转写误路由到流式转写

用户提示词：

```text
转写这个音频文件：./sample.wav
```

预期行为：

- 默认应选择 `stt`
- 只有用户明确要求实时 / streaming / interim results 时才选择 `stt-stream`

## 参数级补测矩阵

这组用例用于确认 `SKILL.md` 只保留常用参数，而 `references` 接住完整参数细节。默认都按 `--json` 理解；认证类命令如果用户显式给了本地文件，就应补上 `--auth-file`。

| 编号 | 命令 | 重点参数 | 用户在 Codex / Claude Code 中可直接说的话 | 预期 skill 路由 |
| --- | --- | --- | --- | --- |
| P1 | `login` | `--no-browser` `--manual-paste` `--timeout` `--port` | 这台机器不能自动开浏览器，帮我手动粘贴方式登录 Grok，超时 300 秒，端口 8787 | `grok-cli login --no-browser --manual-paste --timeout 300 --port 8787` |
| P2 | `status` / `state` | `--auth-file` | 用这个 auth 文件看一下登录状态：`./tmp/auth.json` | `grok-cli status --json --auth-file ./tmp/auth.json` |
| P3 | `model` | `--model` | 把默认文本模型切到 `grok-4.3` | `grok-cli model --json --model grok-4.3` |
| P4 | `usage` | `--session-db` `--session-id` | 从这个 session 数据库里看一下 `abc123` 这条会话的 usage：`./session.db` | `grok-cli usage --json --session-db ./session.db --session-id abc123` |
| P5 | `chat` | `--system` `--model` `--no-web-search` `--with-x-search` `--allowed-domain` `--excluded-domain` `--allowed-x-handle` `--excluded-x-handle` `--from-date` `--to-date` `--enable-image-understanding` `--enable-video-understanding` `--timeout` | 用 Grok 只看 X 和指定域名的资料，限制到 `example.com`，排除 `blocked.example.com`，并只看 `@xAI` 和 `@grok`，时间范围是 2026-05-01 到 2026-05-21，并打开图片和视频理解，返回结构化结果 | `grok-cli chat --json --system "..." --model ... --no-web-search --with-x-search --allowed-domain example.com --excluded-domain blocked.example.com --allowed-x-handle xAI --allowed-x-handle grok --from-date 2026-05-01 --to-date 2026-05-21 --enable-image-understanding --enable-video-understanding --timeout ... --prompt "..."` |
| P6 | `chat` | `--stream` `--no-stream` `--raw-stream` | 我要边生成边看，但不要最终再汇总一次 | `grok-cli chat --stream ...` 或 `grok-cli chat --no-stream ...` 或 `grok-cli chat --raw-stream ...` |
| P7 | `search` | `--query` `--model` `--allowed-x-handle` `--excluded-x-handle` `--from-date` `--to-date` `--enable-image-understanding` `--enable-video-understanding` `--timeout` | 搜索 X 上 `@xAI` 和 `@grok` 相关的讨论，只看最近一周，并打开视频理解 | `grok-cli search --json --query "..." --allowed-x-handle xAI --allowed-x-handle grok --from-date ... --to-date ... --enable-video-understanding --timeout ...` |
| P8 | `image` | `--count` `--response-format` `--output-file` `--output-dir` `--aspect-ratio` `--resolution` `--model` `--timeout` | 生成 3 张 1:1 的终端风格图，输出到目录，并用 base64 保存到本地 | `grok-cli image --json --prompt "..." --count 3 --output-dir ./out --aspect-ratio 1:1 --resolution 1k --response-format b64_json --model ... --timeout ...` |
| P9 | `image-edit` | repeat `--image` `--response-format` `--output-file` `--aspect-ratio` `--resolution` `--model` `--timeout` | 把这两张图融合成一个更像 CLI 封面的图：`./a.png ./b.png` | `grok-cli image-edit --json --image ./a.png --image ./b.png --prompt "..." --response-format b64_json --output-file ./out.png --aspect-ratio 16:9 --resolution 1k --model ... --timeout ...` |
| P10 | `video` | `--image-url` `--image` `--reference-image-url` `--reference-image` `--duration` `--aspect-ratio` `--resolution` `--model` `--timeout` | 用这张本地图片做一个 8 秒视频，再试一个用两张参考图生成的版本 | `grok-cli video --json --prompt "..." --image ./source.png --duration 8 --aspect-ratio 16:9 --resolution 720p --model ... --timeout ...` |
| P11 | `video-edit` | `--video-url` `--video` `--model` `--timeout` | 直接编辑这个本地视频，让画面更有电影感：`./source.mp4` | `grok-cli video-edit --json --video ./source.mp4 --prompt "..." --model ... --timeout ...` |
| P12 | `video-extend` | `--video-url` `--duration` `--model` `--timeout` | 把这个远程视频延长两秒：`https://example.com/source.mp4` | `grok-cli video-extend --json --video-url https://example.com/source.mp4 --duration 2 --prompt "..." --model ... --timeout ...` |
| P13 | `tts` | `--list-voices` `--text` `--voice-id` `--language` `--output` `--output-format` `--sample-rate` `--bit-rate` `--optimize-streaming-latency` `--text-normalization` `--model` `--timeout` | 先列出可用声音，再把这段文字转成 ara 声音的 mp3 并保存 | `grok-cli tts --json --list-voices` 和 `grok-cli tts --json --text "..." --voice-id ara --language en --output ./out.mp3 --output-format mp3 --sample-rate ... --bit-rate ... --optimize-streaming-latency ... --text-normalization ... --model ... --timeout ...` |
| P14 | `stt` | `--file` `--url` `--model` `--language` `--format` `--audio-format` `--sample-rate` `--multichannel` `--channels` `--diarize` `--keyterm` `--filler-words` `--timeout` | 转写这个本地音频，并保留关键词 Grok 和 CLI：`./sample.wav` | `grok-cli stt --json --file ./sample.wav --keyterm Grok --keyterm CLI --language auto --format true --audio-format wav --sample-rate 16000 --multichannel --channels 0,1 --diarize --filler-words --model ... --timeout ...` |
| P15 | `stt-stream` | `--file` `--model` `--language` `--interim-results` `--endpointing` `--encoding` `--sample-rate` `--diarize` `--filler-words` `--multichannel` `--channels` `--keyterm` `--timeout` | 用实时方式转写这个音频，并打开 interim results：`./sample.wav` | `grok-cli stt-stream --json --file ./sample.wav --interim-results --endpointing ... --encoding pcm_s16le --sample-rate 16000 --diarize --filler-words --multichannel --channels 0,1 --keyterm Grok --model ... --language auto --timeout ...` |

## 验收标准

- 所有公开能力至少各有 1 条可执行的 skill 用例。
- 需要本地文件的能力都能通过“直接给路径”的方式触发正确命令。
- `video-extend` 的本地 path 限制会被 skill 明确说明。
- 参数级补测矩阵覆盖 `login`、`chat`、`search`、`image`、`image-edit`、`video`、`video-edit`、`video-extend`、`tts`、`stt`、`stt-stream` 的全部公开参数。
- Codex 和 Claude Code 都可以只依据 `SKILL.md` 与这份用例文档完成验证。
