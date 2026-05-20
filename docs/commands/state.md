# `grok-cli state`

## 用途

查看本地 OAuth 状态文件的脱敏摘要。`state` 不会调用 xAI 网络接口，只读取本地 `auth.json`。

如果只想知道当前是否可用，优先使用 [`status`](./status.md)。`state` 主要用于排障时查看本地保存了什么。

## 常用方式

```bash
grok-cli state
grok-cli state --json
```

检查指定状态文件：

```bash
grok-cli state --auth-file ./auth.json --json
```

## 参数

- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。

## 行为规格

- `state` 直接等价于脱敏 show，不再提供 `path`、`show`、`validate` 子命令。
- 不会打印完整 token，只显示 token 是否存在、过期状态、provider、auth mode、base URL、last refresh、last auth error 等安全摘要。
- 状态文件缺失时，返回 `exists: false`，不会自动创建文件。
- JSON 无法解析或 schema 不满足当前 CLI 需求时，返回 `state_file_invalid`。

## JSON 输出重点

`data` 包含：

- `exists`
- `path`
- `state`

## 相关文档

- [status](./status.md)
- [login](./login.md)
- [样例输出](../reference/samples.md)
