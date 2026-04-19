## Why

The current download flow mixes system proxy behavior, raw-file prefix proxy behavior, and GitHub metadata API fallback in a way that is hard to reason about and produces misleading output. We need to make the raw download path explicit, surface concise failure reasons, and add a permanent debug mode so request construction can be inspected without editing the code.

## What Changes

- Simplify user-facing download failure output so transient raw download failures report a short reason instead of a vague "direct file URL unavailable" message.
- Add an explicit prefix-proxy strategy with `direct`, `fallback`, and `prefer` modes, and scope prefix proxy usage only to raw file download URLs.
- Stop deriving behavior from detected system proxy variables at the CLI output layer; let the HTTP client honor environment defaults without product-level proxy detection or messaging.
- Add a permanent `--debug` mode so URL construction, request strategy choices, and retry flow can be printed when needed without polluting normal output.
- Update bilingual documentation and OpenSpec requirements to describe the new prefix strategy, raw-only proxy scope, and debug mode.

## Capabilities

### New Capabilities
- `prefix-proxy-mode`: Configure whether raw file downloads use a URL-prefix proxy directly, as fallback, or not at all.
- `debug-download-flow`: Print detailed request construction and strategy selection when explicitly enabled.

### Modified Capabilities
- `github-path-download`: Clarify raw download URL selection, concise error reporting, debug output behavior, and the rule that GitHub metadata API requests never use URL-prefix fallback proxies.

## Impact

- Affected code: `src/cli.rs`, `src/download.rs`, `src/output.rs`, and possibly `src/error.rs`
- Affected docs: `README.md`, `README.zh.md`
- Affected specs: `openspec/specs/github-path-download/spec.md`
- Validation: `cargo fmt`, `cargo test`
