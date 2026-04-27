## Why

`gh-download` 已经具备基础开源发布能力，但 GitHub 仓库入口还缺少协作说明、安全报告入口、Issue/PR 模板和更直接的 README 首屏信息。对一个简单 CLI 项目来说，这些文件可以用很低维护成本提高新用户判断效率，也能让外部反馈更完整。

## What Changes

- 增加版本化维护的开源协作入口文件，包括贡献指南、支持说明、安全报告说明、行为准则和 Issue/PR 模板。
- 优化 `README.md` 和 `README.zh.md` 的首屏信息，加入状态徽章、最短安装入口和最短使用示例，并强调 Rust 单文件 CLI、目录下载、私有仓库和多语言输出定位。
- 保持已有安装方式，不新增 Homebrew、Scoop、Winget 等发布渠道。
- 不调整 GitHub 仓库设置，不处理分支保护、Wiki、Discussions、Dependabot 开关等只能在仓库设置中完成的事项。
- 扩展 GitHub Release 打包内容，让预编译二进制归档同时包含 `README.zh.md`。

## Capabilities

### New Capabilities

- `open-source-entrypoints`: 定义仓库内开源协作文件、Issue/PR 模板和 README 首屏信息的维护要求。

### Modified Capabilities

- `binary-release-publishing`: 扩展 Release 归档内容，要求预编译二进制包同时包含英文 README、中文 README 和许可证。

## Impact

- 仓库协作文件：`CONTRIBUTING.md`、`SECURITY.md`、`SUPPORT.md`、`CODE_OF_CONDUCT.md`
- GitHub 模板：`.github/ISSUE_TEMPLATE/*.yml`、`.github/PULL_REQUEST_TEMPLATE.md`
- 用户文档：`README.md`、`README.zh.md`
- 发布工作流：`.github/workflows/release.yml`
- OpenSpec：新增 `open-source-entrypoints`，更新 `binary-release-publishing`
