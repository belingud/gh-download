## Why

`gh-download` currently produces only human-readable terminal output, which makes it awkward to integrate into scripts, CI pipelines, and AI-driven tooling that need structured results. We need an explicit JSON mode so automation can consume stable success and failure data without scraping colored text.

## What Changes

- Add an explicit `--json` mode that emits machine-readable success and failure output for file and directory downloads.
- Define a stable JSON result shape that includes the resolved saved path and download statistics relevant to automation.
- Keep the current human-readable output as the default mode when `--json` is not enabled.
- Clarify how JSON mode interacts with debug output and user-facing error reporting.
- Update bilingual documentation and product specs to describe the new structured output contract.

## Capabilities

### New Capabilities
- `json-output`: Define the machine-readable output contract for success and failure cases.

### Modified Capabilities
- `github-path-download`: Extend the CLI contract and output behavior to cover `--json` mode for download results and failures.
- `debug-download-flow`: Clarify whether debug diagnostics remain separate from JSON output and how they are emitted when both modes are enabled.

## Impact

- Affected code: `src/cli.rs`, `src/cli/types.rs`, `src/cli/help.rs`, `src/output.rs`, `src/error.rs`, `src/main.rs`, and related tests
- Affected docs: `README.md`, `README.zh.md`, `ROADMAP.md`, `ROADMAP.zh.md` if roadmap status needs to reflect progress
- Affected specs: new `openspec/specs/json-output/spec.md`, updates to `openspec/specs/github-path-download/spec.md`, and updates to `openspec/specs/debug-download-flow/spec.md`
- Behavioral impact: automation can rely on a structured output mode without changing the default terminal UX
