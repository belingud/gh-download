## 1. CLI And Configuration

- [x] 1.1 Add an explicit prefix proxy mode to `src/cli.rs` with `direct`, `fallback`, and `prefer`, including CLI/env precedence and tests
- [x] 1.2 Add a `--debug` flag and matching environment control for flow-level debug output, including parsing and precedence tests
- [x] 1.3 Remove product-level system proxy detection/output so ambient proxy env vars remain an HTTP-client concern only

## 2. Download Flow And Output

- [x] 2.1 Refactor `src/download.rs` so GitHub metadata API requests never use URL-prefix proxying and raw file downloads follow the selected prefix mode
- [x] 2.2 Update raw-file warning/error messaging to report short failure-stage reasons and print the full generated prefix URL when prefix retry is used
- [x] 2.3 Implement `--debug` output for generated metadata URLs, resolved raw download URLs, generated prefix URLs, and selected raw download strategy without changing download behavior

## 3. Documentation And Verification

- [x] 3.1 Update `README.md` and `README.zh.md` for the new prefix proxy mode, debug mode, and raw-only proxy scope
- [x] 3.2 Sync `openspec/specs/github-path-download/spec.md` as needed during implementation and keep new capability docs aligned
- [x] 3.3 Add or update unit tests in `src/download.rs`, `src/cli.rs`, and related modules for direct/fallback/prefer behavior, debug output, and concise failure messaging
- [x] 3.4 Run `cargo fmt` and `cargo test`
