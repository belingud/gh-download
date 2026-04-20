## Context

这个 change 同时覆盖两个独立但都属于产品合同层面的缺口。

第一，下载运行时已经存在 `RuntimeConfig.api_base` 和 `DEFAULT_GITHUB_API_BASE`，测试里也会注入自定义 API server，但 CLI 还没有把这个能力公开给用户。这导致公共 GitHub 之外的环境只能依赖改代码或测试注入，不能通过稳定 CLI 接口使用。

第二，仓库目前只有 tag 驱动的 release workflow，缺少面向日常 `push` 和 `pull_request` 的常规验证路径。结果是产品行为回归更依赖本地习惯，而不是仓库层面的自动校验。

这两个变化都会触及跨模块行为：CLI 参数解析、帮助文案、下载请求构造、文档，以及 GitHub Actions workflow 设计，因此需要显式 design 约束实现边界。

## Goals / Non-Goals

**Goals:**
- 增加显式 `--api-base <url>` CLI 选项，让 metadata API 请求可定向到 GitHub Enterprise 或兼容 API base。
- 保持现有下载架构和产品边界不变：metadata-first 检测、raw 下载流式写盘、prefix proxy 仅限 raw 路径、认证请求不走公共代理。
- 增加一个独立于 release workflow 的常规 CI workflow，在 `push` 和 `pull_request` 上执行仓库标准校验。
- 让中英文帮助文本与 README 对新选项和新 workflow 行为保持一致。

**Non-Goals:**
- 不重构下载主流程，不把所有 endpoint 配置扩展成一套全局环境变量体系。
- 不改变 raw 下载 URL、prefix proxy 模式或 token 选择优先级。
- 不把 release workflow 与常规 CI 合并成单一 workflow。
- 不在这次 change 中引入额外 lint、覆盖率上报或跨平台大矩阵常规 CI。

## Decisions

### 1. `--api-base` 只控制 GitHub metadata API base，并保持显式 CLI 优先

CLI 将增加 `--api-base <url>`，解析后进入 `ResolvedOptions`，再在 `run_cli` 中用于构造 `RuntimeConfig`。默认值继续是当前内置的 `https://api.github.com`。

Why:
- 这与现有实现结构直接对齐，改动小，行为明确。
- 用户需要的是公开、稳定、可文档化的入口，而不是隐藏配置点。
- 显式 CLI 选项比新增环境变量更容易在脚本、CI 和故障排查中解释。

Alternatives considered:
- 新增 `GH_DOWNLOAD_API_BASE` 环境变量：放弃，因为会引入新的隐式配置面，且当前产品合同更偏向显式下载参数。
- 同时增加 raw base 配置：放弃，因为当前需求只针对 metadata API，且 raw 行为已有独立边界和代理规则。

### 2. 对 `--api-base` 做轻量规范化，但不做过度兼容推断

实现上只会做与现有 URL 构造兼容的轻量处理，例如去除首尾空白、在拼接时移除末尾 `/`。不会自动推断 `/api/v3`、不会根据 host 猜测 GitHub Enterprise 规则，也不会默默改写用户提供的 host。

Why:
- 当前 `build_contents_api_url` 已经基于 `trim_end_matches('/')` 构造 URL，说明轻量规范化足够。
- 自动猜测 enterprise 路径容易把错误配置掩盖成难排查的 404/401。

Alternatives considered:
- 自动补齐 enterprise 常见路径：放弃，因为这会把显式合同变成启发式行为。
- 完全不校验或规范化：放弃，因为保留基本的空白处理和 trailing slash 兼容更稳妥。

### 3. 调试输出必须反映实际使用的 metadata URL

启用 `--debug` 时，现有 `metadata-url` 诊断输出应直接展示 `--api-base` 生效后的完整 URL。这样在公共 GitHub 与自定义 API base 间切换时，用户能立即确认请求到底打到了哪里。

Why:
- `--api-base` 的主要失败模式就是 base 配置错误，debug 输出是最直接的排错入口。
- 这与现有 debug contract 一致，不需要引入新的输出通道。

Alternatives considered:
- 仅在帮助文档中说明，不在 debug 中显式体现：放弃，因为排错信息不足。

### 4. 常规 CI 使用单独 workflow，执行仓库标准 `just check`

新增一个常规验证 workflow，触发条件为普通 `push` 和 `pull_request`。workflow 使用单一 Linux runner 安装稳定 Rust 后执行 `just check`，并保留现有 `release.yml` 只负责 tag 发布构建与发布。

Why:
- `just check` 已经是仓库约定的标准校验入口，避免在 CI 中复制另一套命令编排。
- 单独 workflow 能让发布路径与日常验证路径职责清晰，维护成本低。

Alternatives considered:
- 直接把 `push` / `pull_request` 触发加到 `release.yml`：放弃，因为会混淆发布与常规验证职责。
- 在常规 CI 中跑完整发布矩阵：放弃，因为当前目标是基础护栏，不是扩大 CI 成本。

## Risks / Trade-offs

- [`--api-base` 配错时会直接请求错误 endpoint] -> 通过帮助文本、README 示例和 debug URL 输出降低排查成本。
- [只做 `just check` 的单平台 CI 无法覆盖所有 release target] -> 明确保留 release workflow 负责发布矩阵，常规 CI 只承担快速回归检测。
- [内部已有 `RuntimeConfig.api_base`，实现时容易遗漏 CLI 到 runtime 的接线] -> 用解析测试和下载 URL 测试覆盖从 CLI 到请求构造的链路。
- [文档与 spec 若不同步，会让企业环境支持变得模糊] -> 这次 change 明确要求同步更新中英文 README 和相关 specs。

## Migration Plan

1. 修改 OpenSpec：扩展 `github-path-download` 的 CLI 合同，并新增 `regular-ci-validation` capability。
2. 在 CLI 类型、解析和帮助文案中加入 `--api-base`，默认仍指向公共 GitHub API。
3. 将解析后的 `api_base` 接入 `RuntimeConfig` 创建路径，并补充针对自定义 API base 的测试。
4. 新增常规 GitHub Actions workflow，在 `push` 和 `pull_request` 上执行 `just check`。
5. 更新 README.md 与 README.zh.md，明确 `--api-base` 用途、常规 CI 行为，以及 release workflow 与 regular CI 的职责边界。
6. 运行 `just fmt`、`just test`、`just check` 进行验证。

Rollback strategy:
- 如果 `--api-base` 的外部合同在实现中暴露出更多兼容性问题，可在发布前移除该 flag，保留内部 runtime 配置点不对外承诺。
- 如果常规 CI 初次接入暴露环境不稳定问题，可先保留 workflow 结构并缩小触发范围，但不改动 release workflow。

## Open Questions

- None currently.
