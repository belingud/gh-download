## 1. CLI Contract And Runtime Wiring

- [x] 1.1 在 `src/cli/types.rs`、`src/cli/resolve.rs`、`src/cli/help.rs` 中增加 `--api-base` 选项、解析逻辑和中英文帮助文本，并补充相关解析测试。
- [x] 1.2 在 `src/lib.rs` 中把解析后的 `api_base` 接入 `RuntimeConfig`，确保下载运行时实际使用 CLI 提供的 metadata API base。
- [x] 1.3 在下载相关测试中覆盖自定义 `api_base` 生效后的 metadata URL 构造与调试输出链路，确保不影响现有 raw 下载和代理边界。

## 2. Regular CI Workflow

- [x] 2.1 新增常规 GitHub Actions workflow，在普通 `push` 和 `pull_request` 事件上触发验证。
- [x] 2.2 在该 workflow 中安装稳定 Rust 和 `just`，并执行 `just check` 作为仓库标准校验命令。
- [x] 2.3 确认常规 CI 与现有 `.github/workflows/release.yml` 职责分离，不引入打包或发布步骤。

## 3. Docs And Verification

- [x] 3.1 更新 `README.md` 与 `README.zh.md`，说明 `--api-base` 的用途、示例，以及常规 CI 与 release workflow 的区别。
- [x] 3.2 运行 `just fmt`、`just test` 与 `just check`，验证 CLI 改动和 workflow 相关改动没有破坏现有行为。
