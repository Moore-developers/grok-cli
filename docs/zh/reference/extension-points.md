# 后续能力扩展接口

这份文档用于收口 `grok-cli` 的后续扩展方向，帮助后续继续新增能力时，不破坏已经稳定的 JSON 输出、错误码和 SKILL 集成边界。

## 1. 当前已经稳定的扩展边界

下面这些边界已经在当前实现里形成稳定约定，后续新增能力时应尽量复用，而不是重开一套旁路：

### 1.1 命令分层

- 认证命令：`login`、`status`、`refresh`、`logout`
  - 负责 OAuth 与 runtime credential 准备
- 内部认证救援入口：隐藏的 `exchange-code`
  - 只用于 loopback / manual-paste 异常场景，不属于公开日常命令面
- `state`
  - 负责状态文件路径、校验、脱敏展示
- 能力命令：`chat`、`search`、`image`、`video`、`tts`、`stt`
  - 负责确定性能力执行

扩展建议：
- 新能力优先做成新的顶层命令
- 新的鉴权流程优先复用现有认证命令，而不是恢复旧的二级命令分组

### 1.2 统一输出契约

成功信封：

```json
{
  "ok": true,
  "command": "chat",
  "data": {}
}
```

失败信封：

```json
{
  "ok": false,
  "command": "chat",
  "error": {
    "code": "request_failed",
    "message": "...",
    "relogin_required": false,
    "entitlement_denied": false
  }
}
```

扩展建议：
- 新能力优先保留当前统一信封
- 不要为单个能力发明独立顶层 JSON 结构

### 1.3 统一运行时凭据

当前所有真实能力都通过共享 runtime credentials 解析：

- 统一读取状态文件
- 统一判断 access token 是否可复用
- 统一按需 refresh
- 统一输出 `provider / base_url / bearer`

扩展建议：
- 新能力不要自行读取 token
- 新能力不要绕开共享 resolver 直接拼 `Authorization`

### 1.4 共享 upstream 执行层

当前已经有四类可复用的上游执行入口：

- JSON `POST`
- JSON `GET`
- 二进制 `POST`
- multipart `POST`
- `responses` SSE stream

扩展建议：
- 如果后续只是新增一个 JSON 型能力，先在现有 upstream 上复用
- 只有当上游协议真的不同，才新增新的执行形态

## 2. 推荐的后续扩展方向

### 2.1 顶层能力命令扩展

优先级较高的继续扩展项：

- 更完整的 `chat`
  - `previous_response_id`
  - 多轮上下文拼接
  - 工具调用结果继续回注
- 更细的 `image` / `video`
  - 更多模型参数
  - 更细的失败原因分类
- 更完整的 `tts` / `stt`
  - 更细的模型、语言、音频格式控制

### 2.2 文档与验收扩展

后续可以继续补：

- 真实浏览器认证录屏或截图说明
- 更多 capability 的端到端验收脚本

## 3. 新能力接入建议流程

如果后续要新增一个能力，推荐按下面顺序推进：

1. 先明确命令挂载位置
2. 明确它用哪一种 upstream 执行形态
3. 明确成功输出主字段
4. 明确失败时复用哪些错误码
5. 先补模块级解析测试
6. 再补命令级 stub 测试
7. 最后补文档与样例输出

## 4. 当前不建议轻易改动的稳定面

这些内容已经被文档、样例和回归测试依赖，后续不要轻易改：

- 顶层 JSON 成功 / 失败信封
- 现有错误码字符串
- `chat` 的 `protocol / output_text / finish_reason / tool_calls`
- `search` 的 `answer / citations / inline_citations`
- `image` 的 `image`
- `video` 的 `video`
- `tts` 的 `file_path`
- `stt` 的 `transcript`

## 5. 扩展时的回归要求

后续每增加一个真实能力，建议至少同时补三类东西：

- 一组模块级解析测试
- 一组命令级 stub 测试
- 一段文档样例

这样可以保证：
- 代码可维护
- 输出契约可回归
- SKILL 集成层不会在升级时悄悄漂移
