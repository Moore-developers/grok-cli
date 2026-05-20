# `grok-cli logout`

## 用途

删除本地 OAuth 状态文件，让 CLI 回到未登录状态。

## 常用方式

```bash
grok-cli logout
```

脚本或 SKILL：

```bash
grok-cli logout --json
```

## 参数

- `--json`：使用统一 JSON 信封输出。
- `--auth-file <PATH>`：覆盖 OAuth 状态文件路径。

## 行为规格

- 删除指定或默认的 `auth.json`。
- 如果文件不存在，命令仍然成功，并返回 `removed: false`。
- 不删除 session usage SQLite 数据库。

## JSON 输出重点

`data` 中包含：

- `removed`
- `auth_store_path`

## 相关文档

- [login](./login.md)
- [state](./state.md)

