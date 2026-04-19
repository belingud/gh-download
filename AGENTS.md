# AGENTS.md

## Project Overview

`gh-download` is a single-binary Rust CLI for downloading either a file or a directory from a GitHub repository path.

The current product contract is primarily defined by these locations:

- `src/`: runtime code
- `README.md` / `README.zh.md`: bilingual user documentation
- `openspec/specs/github-path-download/spec.md`: main contract for CLI and download behavior
- `openspec/specs/binary-release-publishing/spec.md`: main contract for release and binary packaging behavior
- `.github/workflows/release.yml`: the actual tag-triggered release pipeline

This repository is no longer a "script repo". It is now a product repo centered on a Rust CLI. If you change user-visible behavior, do not update code alone. You must also sync the docs and specs.

## Repository Map

- `Cargo.toml`: crate metadata, dependencies, and package include rules; the current version is maintained here.
- `Cargo.lock`: locked dependency versions; it changes when dependencies change or during release prep.
- `Justfile`: common local commands plus the `prepare-release` / `publish` release helper flows.
- `.gitignore`: ignores `target/`, debug artifacts, and mutation testing artifacts.
- `.github/workflows/release.yml`: when a `v*` tag is pushed, builds 5 platform targets, packages archives, generates `checksums.txt`, and publishes a GitHub Release.
- `src/main.rs`: CLI entrypoint; parses arguments, detects language, checks whether a token is present, and prints user-facing errors on failure.
- `src/lib.rs`: module export layer; wires the CLI, output layer, and download runner together.
- `src/cli.rs`: `clap` argument definitions and parsing; handles empty-invocation help, localized `--lang` help, token/proxy/env precedence, and local path expansion.
- `src/download.rs`: download core; calls the GitHub Contents API, recursively enumerates directories, streams files to disk, handles anonymous proxy fallback, and contains download tests.
- `src/error.rs`: internal error type `AppError` and user-facing error type `UserFacingError`, including localized suggestions in Chinese and English.
- `src/output.rs`: formatting for startup summaries, scan progress, download progress, warnings, success output, and error output.
- `src/i18n.rs`: language enum and locale detection logic; precedence is `--lang` > `LC_ALL` > `LC_MESSAGES` > `LANG`.
- `README.md`: English documentation.
- `README.zh.md`: Chinese documentation.
- `openspec/config.yaml`: repository-level OpenSpec rules that explicitly treat user-visible behavior as part of the product contract.
- `openspec/specs/*/spec.md`: currently active specs.
- `openspec/changes/archive/*`: archived proposal/design/tasks/delta specs, mainly useful for tracing design history.

## Current Behavior Contract

- The CLI defaults to English; it switches to Chinese when the locale indicates Chinese; `--lang` overrides locale detection.
- Running `gh-download` with no user-provided arguments should show help in the effective language and exit successfully.
- The required positional arguments remain `<repo> <remote-path> <local-target>`; partially missing arguments should still follow `clap`'s error behavior.
- Token precedence is fixed as `--token` > `GITHUB_TOKEN` > `GH_TOKEN`.
- Proxy fallback is only for anonymous requests; authenticated requests must not forward credentials to a public proxy.
- Directory downloads must preserve the relative path structure; file downloads must support both "write to a file path" and "write into an existing directory".
- Downloads should continue to stream to disk; do not change the implementation to fully buffer files in memory.
- User-facing output in English and Chinese should remain semantically aligned; do not update only one language.

## Change Coupling Rules

- When changing CLI arguments, help text, defaults, or language behavior:
  - Update `src/cli.rs`
  - Check `src/i18n.rs`
  - Update both `README.md` and `README.zh.md`
  - Update `openspec/specs/github-path-download/spec.md`
  - Add or adjust the relevant tests
- When changing download semantics, path handling, proxy strategy, or file save rules:
  - Focus on `src/download.rs`
  - Update `src/output.rs` and `src/error.rs` when needed
  - Update `openspec/specs/github-path-download/spec.md`
  - Add tests with `mockito` / `tempfile` and avoid relying on live GitHub
- When changing error messages or terminal output:
  - Check `src/output.rs` and `src/error.rs`
  - Update example output in the bilingual README files
- When changing release flow, platform matrix, asset naming, or packaging behavior:
  - Update `.github/workflows/release.yml`
  - Update `openspec/specs/binary-release-publishing/spec.md`
  - Update the release helper notes in `Justfile` when needed

## Development And Verification

- Common formatting: `just fmt`
- Common tests: `just test`
- Common checks: `just check`

Testing guidance:

- Prefer local, repeatable tests and avoid relying on real network access.
- `src/download.rs` already uses `mockito` + `tempfile` to cover single-file downloads, recursive directory downloads, and proxy fallback.
- `src/cli.rs`, `src/i18n.rs`, and `src/error.rs` already contain unit tests for parsing, language handling, and error classification; add new tests close to the changed behavior when possible.

## Maintenance Notes

- The version does not appear only in `Cargo.toml`; `src/download.rs` also contains a hard-coded `USER_AGENT_VALUE`, so verify it before release.
- `Justfile`'s `prepare-release` checks that the working tree is clean, updates `Cargo.toml`, regenerates `Cargo.lock`, runs `fmt`/`test`, and creates a local tag.
- Historical implementation context from archived OpenSpec changes can help explain design background, but the real current contract should come from `openspec/specs/` and the existing source code.

## Advice For Agents

- Start with `src/cli.rs` and `src/download.rs`, then decide whether `output`, `error`, README files, and OpenSpec also need updates.
- If a change affects user-visible behavior, assume the bilingual docs and specs need updating too rather than changing code alone.
