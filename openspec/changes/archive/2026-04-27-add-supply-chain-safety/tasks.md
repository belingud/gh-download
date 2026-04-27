## 1. Dependabot 配置

- [x] 1.1 新增 `.github/dependabot.yml`，配置 Cargo 依赖 weekly 更新
- [x] 1.2 在 `.github/dependabot.yml` 中配置 GitHub Actions 依赖 weekly 更新
- [x] 1.3 为 Dependabot 配置合理的 PR 数量限制，并保留默认标签行为

## 2. Rust 依赖审计工作流

- [x] 2.1 新增 `.github/workflows/audit.yml`
- [x] 2.2 让审计工作流在 Cargo 依赖文件或自身变更的 push/pull_request 中运行
- [x] 2.3 让审计工作流支持每周定时运行和手动触发
- [x] 2.4 保持常规 CI 不变，不把审计步骤加入 `ci.yml`

## 3. 验证

- [x] 3.1 运行 OpenSpec 校验，确认新规格可解析
- [x] 3.2 校验新增 YAML 文件可解析
- [x] 3.3 运行 `just fmt`
- [x] 3.4 运行 `just test`
- [x] 3.5 运行 `just check`
