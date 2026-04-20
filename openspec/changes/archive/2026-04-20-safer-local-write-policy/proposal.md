## Why

The current download flow overwrites existing local files implicitly, which is risky for repeated runs and makes the CLI harder to use safely in automation. We need a safer default that preserves existing files unless the user explicitly opts into replacement.

## What Changes

- **BREAKING** Change the default local write behavior so existing local files are skipped instead of overwritten.
- Add an explicit `--overwrite` flag to allow users to replace existing local files during file and directory downloads.
- Keep directory structure resolution, metadata-first detection, streaming writes, and proxy/authentication boundaries unchanged.
- Update user-facing output and bilingual documentation so skipped-versus-overwritten outcomes are clear.

## Capabilities

### New Capabilities
- `local-write-policy`: Define how the CLI handles existing local files, including the default skip behavior and explicit overwrite mode.

### Modified Capabilities
- `github-path-download`: Extend the download contract and output behavior to cover skipped existing files and explicit overwrite requests for file and directory downloads.

## Impact

- Affected code: `src/cli.rs`, `src/cli/types.rs`, `src/cli/help.rs`, `src/download.rs`, `src/download/transport.rs`, `src/output.rs`, and related tests
- Affected docs: `README.md`, `README.zh.md`, `ROADMAP.md`, `ROADMAP.zh.md` if release planning text needs to reflect progress
- Affected specs: new `openspec/specs/local-write-policy/spec.md` and updates to `openspec/specs/github-path-download/spec.md`
- Behavioral impact: existing automation that relied on implicit overwrite will need to pass `--overwrite`
