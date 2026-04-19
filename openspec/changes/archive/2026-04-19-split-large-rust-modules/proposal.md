## Why

`src/download.rs` and `src/cli.rs` now contain most of the product logic and tests in very large files, which makes targeted changes harder to review and increases the risk of accidental behavior changes. This change reorganizes the Rust code into smaller modules while preserving the current CLI and download contract.

## What Changes

- Split oversized Rust modules into focused submodules for CLI parsing/resolution, download orchestration, HTTP transport, path utilities, raw-download strategy handling, and related tests.
- Preserve the existing public library exports needed by tests and callers, including current helper function names where practical.
- Keep all user-visible behavior unchanged: arguments, defaults, locale behavior, output wording, error classification, proxy behavior, path handling, and release behavior remain the same.
- Move or reorganize tests alongside the modules they cover, adding regression coverage only where needed to prove behavior was preserved.
- Do not introduce new runtime dependencies or change the binary packaging flow.

## Capabilities

### New Capabilities

- `code-organization`: Internal maintainability constraints for splitting large Rust modules without changing product behavior.

### Modified Capabilities

None. This is an internal refactor and does not change the requirements of `github-path-download`, `prefix-proxy-mode`, `debug-download-flow`, or `binary-release-publishing`.

## Impact

- Affected code: primarily `src/download.rs`, `src/cli.rs`, and `src/lib.rs`; possibly test-only module files under `src/`.
- Affected docs: no README or spec updates are expected because user-visible behavior is intentionally unchanged.
- Affected APIs: the crate's existing public re-exports should remain source-compatible for current tests and local callers.
- Validation: `just fmt`, `just test`, and `just check`.
