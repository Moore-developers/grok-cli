# SKILL 集成约定

## 目标

让上层 SKILL 能稳定依赖 `grok-cli` 的 JSON 输出、错误码和恢复流程，而不是把认证、状态和能力细节散落在多个脚本中。

仓库内置 [`skills/grok-cli/SKILL.md`](../../../skills/grok-cli/SKILL.md)，作为推荐的用户入口。这个 skill 会在可行时直接执行用户请求的原始命令，只在真实失败后处理安装、修复、OAuth 登录或刷新，并在恢复后重试用户的原始 Grok 任务。

## 1. 命令调用原则

- 优先使用 `--json`
- 不要依赖文本命令的默认流式行为
- 默认把标准输出当作机器可消费输出
- 标准错误只用于日志和调试
- 调用前不要假定 CLI 已安装；如果是通过 skill 使用，应先执行安装检查
- 普通用户任务前不要先跑 `status`、登录检查、刷新检查、权限检查或能力探针
- 先执行用户真实命令；只有真实 shell 或 JSON 错误出现后再恢复
- 构造命令时，原样保留用户的 prompt、query、正文、文件路径、URL 和输出要求；只做 shell/CLI 必需的引用、转义、选 flag 或路径解析。

说明：
- `chat` 和 `search` 面向人类交互时默认会流式打印正文
- SKILL、脚本、自动化应固定使用 `--json`
- 只有在调用方明确具备 SSE 解析能力时，才建议使用 `--raw-stream`

## 2. 推荐调用顺序

### 普通能力调用

1. 构造用户的原始命令，例如 `search --json --query "..."`
2. 直接执行它
3. 如果成功，渲染结果
4. 如果失败，根据真实错误恢复，然后重试一次原始命令

## 3. 认证后恢复原始任务

推荐把原始任务参数保存在 SKILL 自己的上下文里，而不是依赖 CLI 内部帮你记住“用户刚才想做什么”。

推荐模式：

1. SKILL 先准备并执行原始命令
2. 如果原始命令失败且提示认证问题，则插入认证流程
3. 认证成功后，重新执行原始命令

示例：

```text
原始任务: search --json --query "..."
真实命令失败: auth_missing
恢复动作: refresh --json
认证成功后重新执行原始任务
```

## 4. 错误码处理建议

- `state_file_missing`
  - 说明还没有认证状态文件
  - 通常进入 `login`
- `auth_missing`
  - 说明当前任务缺少有效凭据
  - 通常先 `refresh --json`，refresh 无法恢复或要求重登时再 `login`
- `bad-credentials` / 过期凭据
  - 通常先 `refresh --json`，再重试原始命令
- `auth_relogin_required`
  - 说明刷新已无法恢复
  - 必须重新登录
- `xai_oauth_tier_denied`
  - 说明 OAuth 账号没有对应能力权限
  - 不要误导用户去重登

## 5. 原样透传输出契约

SKILL 依赖下列字段时，可以认为它们已经稳定：

- 顶层成功信封：`ok` / `command` / `data`
- 顶层失败信封：`ok` / `command` / `error.code` / `error.message`
- `chat`：`protocol` / `output_text` / `finish_reason` / `tool_calls`
- `search`：`answer` / `citations` / `inline_citations`
- `image`：`image`
- `video`：`video`
- `tts`：`file_path`
- `stt`：`transcript`

返回给用户时，默认使用“无损人类可读渲染”：解析 JSON 信封，抽取命令对应的主要字段，并原样展示字段值。安装、登录、刷新等恢复动作只在结果之后用单独的简短说明提及。

- `chat --json`：原样返回 `data.output_text`
- `search --json`：原样返回 `data.answer`，需要展示引用时原样保留 `data.citations` / `data.inline_citations`
- `stt --json`：原样返回 `data.transcript`
- 媒体命令：原样返回 JSON 字段中的路径、URL、request id、media tag 或 handle
- 除非用户明确要求翻译、总结、改写、重排、格式化或分析，否则不要由宿主助手二次加工 Grok 文本
- 如果用户要求 raw CLI output 或 raw JSON，完整返回 stdout/stderr 内容；除了为了安全展示而加代码围栏，不改内容
