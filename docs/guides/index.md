# Guides Index

这个目录放用户和发布维护者最常用的操作指南。它们偏“怎么做”，不承担完整参数规格。

## 阅读顺序

1. [快速开始](./quickstart.md)：从安装、登录到第一次真实调用。
2. [故障排查](./troubleshooting.md)：认证、权限、媒体、stream 等常见问题。
3. [发布与安装指南](./release.md)：GitHub 发布、Cargo 安装、Release binary、Homebrew 等发布路径。

## 文档职责

### [快速开始](./quickstart.md)

适合第一次使用 `grok-cli` 的用户。它只保留最短上手路径：

- 编译和测试
- 查看认证状态
- 发起浏览器登录
- 执行 chat / search / media / usage
- 跳转到更完整的命令参考

### [故障排查](./troubleshooting.md)

适合命令报错时快速定位问题。当前覆盖：

- `state_file_missing`
- `auth_relogin_required`
- `xai_oauth_tier_denied`
- 浏览器授权成功但 CLI 后续失败
- `stt` 文件和参数问题
- `chat --stream` 异常

### [发布与安装指南](./release.md)

适合准备把项目发布给其他用户时使用。当前覆盖：

- 从 GitHub source 安装
- GitHub Release binaries
- Homebrew tap
- crates.io 的后续选择
- 发布前检查清单

## 相关入口

- [CLI 命令索引](../commands/index.md)
- [Reference Index](../reference/index.md)
- [Project Index](../project/index.md)
- [总索引](../index.md)

