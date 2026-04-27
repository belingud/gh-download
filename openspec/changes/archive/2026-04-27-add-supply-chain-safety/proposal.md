## Why

`gh-download` 是面向公开使用的 Rust CLI，依赖和 GitHub Actions 版本需要有持续更新入口。当前仓库没有版本化的 Dependabot 配置，也没有定期检查 RustSec 安全公告的工作流。

## What Changes

- 增加 `.github/dependabot.yml`，让 Dependabot 定期检查 Cargo 依赖和 GitHub Actions 依赖。
- 增加独立的 Rust 依赖审计工作流，在依赖文件或审计工作流变更时运行，并定期运行以捕获新披露的 RustSec 公告。
- 保持常规 CI 的职责不变；`ci.yml` 仍然负责 `just check`。
- 不调整 GitHub 仓库设置，不处理 Dependabot alerts/security updates 开关。
- 不改变 CLI 行为、发布资产或用户文档。

## Capabilities

### New Capabilities

- `supply-chain-safety`: 定义依赖版本更新配置和 Rust 依赖安全审计工作流。

### Modified Capabilities

- None.

## Impact

- GitHub 依赖更新配置：`.github/dependabot.yml`
- GitHub Actions：新增 `.github/workflows/audit.yml`
- OpenSpec：新增 `supply-chain-safety`
