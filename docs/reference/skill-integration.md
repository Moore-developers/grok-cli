# SKILL 集成约定

## 目标

让上层 SKILL 能稳定依赖 `grok-cli` 的 JSON 输出、错误码和恢复流程，而不是把认证、状态和能力细节散落在多个脚本中。

仓库内置 [`skills/grok-cli/SKILL.md`](../../skills/grok-cli/SKILL.md)，作为推荐的用户入口。这个 skill 负责检查本机是否已有 `grok-cli`、缺失时通过 Cargo 从 GitHub 安装、处理 OAuth 登录，并在设置完成后恢复用户的原始 Grok 任务。

## 1. 命令调用原则

- 优先使用 `--json`
- 不要依赖文本命令的默认流式行为
- 默认把标准输出当作机器可消费输出
- 标准错误只用于日志和调试
- 调用前不要假定状态文件存在，先检查 `status` 或 `state --json`
- 调用前不要假定 CLI 已安装；如果是通过 skill 使用，应先执行安装检查

说明：
- `chat` 和 `search` 面向人类交互时默认会流式打印正文
- SKILL、脚本、自动化应固定使用 `--json`
- 只有在调用方明确具备 SSE 解析能力时，才建议使用 `--raw-stream`

## 2. 推荐调用顺序

### 普通能力调用

1. `status --json`
2. 如果缺失或不可用，则进入 `login`
3. 登录完成后恢复原始 top-level capability commands

## 3. 认证后恢复原始任务

推荐把原始任务参数保存在 SKILL 自己的上下文里，而不是依赖 CLI 内部帮你记住“用户刚才想做什么”。

推荐模式：

1. SKILL 先准备要执行的原始命令
2. 如果 `status` 失败或提示要重登，则插入认证流程
3. 认证成功后，重新执行原始命令

示例：

```text
原始任务: search --json --query "..."
前置检查失败: status -> state_file_missing
恢复动作: login --json
认证成功后重新执行原始任务
```

## 4. 错误码处理建议

- `state_file_missing`
  - 说明还没有认证状态文件
  - 通常进入 `login`
- `auth_missing`
  - 说明当前任务缺少有效凭据
  - 通常重新登录或刷新
- `auth_relogin_required`
  - 说明刷新已无法恢复
  - 必须重新登录
- `xai_oauth_tier_denied`
  - 说明 OAuth 账号没有对应能力权限
  - 不要误导用户去重登

## 5. 输出契约建议

SKILL 依赖下列字段时，可以认为它们已经稳定：

- 顶层成功信封：`ok` / `command` / `data`
- 顶层失败信封：`ok` / `command` / `error.code` / `error.message`
- `chat`：`protocol` / `output_text` / `finish_reason` / `tool_calls`
- `search`：`answer` / `citations` / `inline_citations`
- `image`：`image`
- `video`：`video`
- `tts`：`file_path`
- `stt`：`transcript`
