## Context

仓库当前有常规 CI 和 Release 工作流，但缺少两个基础供应链入口：依赖版本更新提醒，以及针对 Rust crate 安全公告的定期审计。GitHub 文档要求 Dependabot 配置文件放在 `.github/dependabot.yml`，并为每个 ecosystem 配置 `package-ecosystem`、`directory` 和 `schedule.interval`。RustSec 提供 Rust crate 安全公告数据库，适合用 `cargo audit` 类工具做定期检查。

本次变更只新增仓库内文件，不修改 GitHub 仓库设置。Dependabot alerts 和 security updates 是否启用仍由仓库设置决定，不在本次实现中处理。

## Goals / Non-Goals

**Goals:**
- 让 Cargo 依赖和 GitHub Actions 依赖有定期版本更新 PR 入口。
- 让 RustSec 安全公告能通过 GitHub Actions 定期检查。
- 保持审计工作流独立于普通 CI，避免每个普通文档变更都触发安全审计。
- 使用少量配置，适合小型 CLI 项目维护。

**Non-Goals:**
- 不引入 `cargo deny` 配置。
- 不启用或修改 GitHub 仓库设置里的 Dependabot alerts、security updates、branch protection。
- 不把安全审计加入 `just check`。
- 不新增本地开发依赖。

## Decisions

### 1. Dependabot 覆盖 Cargo 和 GitHub Actions

`.github/dependabot.yml` 配置两个 ecosystem：

- `cargo`，目录为 `/`
- `github-actions`，目录为 `/`

两者都按 weekly 检查，限制同时打开的版本更新 PR 数量，并保留 Dependabot 默认标签。

Why:
- 这个仓库的依赖面集中在 `Cargo.toml`、`Cargo.lock` 和 `.github/workflows/`。
- weekly 对简单 CLI 项目足够，不会产生太多维护噪声。
- GitHub Actions 依赖也属于供应链面，需要跟随版本更新。
- Dependabot 默认会自动创建并使用依赖相关标签，避免维护自定义标签配置。

Alternatives considered:
- daily：放弃，因为当前项目依赖少，过于频繁。
- 只检查 Cargo：放弃，因为 workflow action 版本也会影响构建和发布安全。

### 2. 新增独立 audit workflow

新增 `.github/workflows/audit.yml`。触发条件：

- `push` 和 `pull_request`，但只在 `Cargo.toml`、`Cargo.lock` 或 audit workflow 自身变更时运行。
- 每周定时运行。
- 支持 `workflow_dispatch` 手动触发。

Why:
- 依赖安全问题可能在依赖没有变化时才被披露，定时运行能捕获这种情况。
- 路径过滤避免文档改动触发额外审计。
- 手动触发便于维护者排查。

Alternatives considered:
- 把审计放进 `ci.yml`：放弃，因为会扩大普通 CI 职责，也会让文档 PR 承担不必要的网络和 advisory 数据库更新成本。
- 使用 `cargo deny`：暂不采用，因为需要更多策略配置；当前先用安全公告审计解决主要风险。

### 3. 使用 RustSec 审计 Action，不新增项目依赖

审计工作流使用现成 GitHub Action 运行 RustSec advisory 检查，不把 `cargo-audit` 加进项目依赖，也不要求开发者本地安装。

Why:
- 供应链检查属于仓库自动化，不是 CLI 运行时能力。
- 避免把维护工具放进 `Cargo.toml`，保持产品依赖面干净。

## Risks / Trade-offs

- [新增审计工作流会依赖网络和 advisory 数据库状态] → 只在依赖相关变更和定时任务中运行，不影响普通 `just check` 合同。
- [Dependabot 可能产生维护噪声] → 使用 weekly，并限制打开 PR 数量。
- [RustSec 审计 Action 本身也是供应链依赖] → 同时用 Dependabot 监控 `github-actions` ecosystem。
- [仓库设置中的 Dependabot alerts 仍可能关闭] → 本次只提交版本化配置；设置项留给仓库维护者处理。

## Migration Plan

1. 新增 `supply-chain-safety` capability。
2. 增加 `.github/dependabot.yml`。
3. 增加 `.github/workflows/audit.yml`。
4. 校验 YAML、OpenSpec、格式和现有测试。

Rollback strategy:
- 如果审计 workflow 噪声过高，可删除 `.github/workflows/audit.yml`，保留 Dependabot 更新配置；如果 Dependabot PR 过多，可调低 `open-pull-requests-limit` 或改为 monthly。

## Open Questions

- None currently.
