## Why

`gh-download` 已经具备发布流程，但还缺少针对日常 `push` 和 `pull_request` 的常规校验，导致回归更容易在发布前后才暴露。与此同时，CLI 目前默认固定使用 GitHub 公共 API base，这限制了它在 GitHub Enterprise 或自托管兼容环境中的可用性。

## What Changes

- 增加 `--api-base <url>`，允许用户显式指定 GitHub metadata API 的基础地址。
- 保持当前产品约束不变：metadata 请求仍然先做远端类型判断，raw 下载与 URL-prefix 代理边界不变，认证信息不能转发到公共代理路径。
- 增加面向普通 `push` 和 `pull_request` 的 GitHub Actions 校验 workflow，执行仓库标准校验命令，并与现有 tag 驱动 release workflow 保持分离。
- 更新中英文文档，说明新的 CLI 选项与 CI 行为。

## Capabilities

### New Capabilities
- `regular-ci-validation`: 定义仓库在普通 `push` 与 `pull_request` 上运行的标准 GitHub Actions 校验流程。

### Modified Capabilities
- `github-path-download`: 扩展下载 CLI 合同，支持通过 `--api-base` 配置 GitHub metadata API 基础地址，同时保持既有下载与代理边界规则。

## Impact

- CLI 参数与帮助文本：`src/cli.rs`、`src/cli/types.rs`、`src/cli/resolve.rs`、`src/cli/help.rs`
- 下载请求构造与 transport：`src/download.rs`、`src/download/transport.rs`
- 用户文档：`README.md`、`README.zh.md`
- CI 与发布配置：新增常规 workflow，并确保与 `.github/workflows/release.yml` 的职责边界清晰
