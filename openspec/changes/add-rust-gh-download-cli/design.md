## Context

The repository currently centers on [download_github_path.py](/Users/vic/Documents/codes/git/gh-download/download_github_path.py:1), a Python script that downloads either a file or a directory from a GitHub repository path using the GitHub Contents API, optional token authentication, and a public proxy fallback. The new change replaces the Python-first delivery model with a compiled Rust CLI while preserving the practical download workflow users already rely on.

This change crosses several concerns at once:

- CLI UX: argument parsing, colored output, concise progress reporting, and actionable failure messages
- Download engine: GitHub metadata requests, file streaming, directory traversal, proxy fallback, and local path resolution
- Distribution: automated multi-platform binary packaging and GitHub Release publishing on version tags

The repository does not currently contain existing specs or a release pipeline, so this change establishes both the product contract and the technical baseline for future iterations.

## Goals / Non-Goals

**Goals:**
- Ship a single Rust binary that can download a file or directory tree from a GitHub repository path.
- Preserve support for `--ref`, `--token`, `GITHUB_TOKEN` / `GH_TOKEN`, and `--proxy-base`.
- Make CLI output easy to scan with colored status lines, short summaries, and concrete remediation hints for common failures.
- Package binaries for major Linux, macOS, and Windows targets and publish them automatically when a `v*` tag is pushed.
- Keep the implementation small enough to maintain in a single repository without adding heavy release tooling.

**Non-Goals:**
- Reproduce the Python script line-for-line or preserve its exact internal structure.
- Implement parallel downloads, resumable downloads, or archive extraction in the first release.
- Add package manager integrations such as Homebrew, Scoop, or Cargo installation automation in this change.
- Support non-GitHub hosts or enterprise GitHub instances in the first version.

## Decisions

### 1. Build a single binary crate named `gh-download`

The repository will become a standard Rust CLI crate with a single executable entrypoint. The tool will expose a simple positional interface:

`gh-download <repo> <remote-path> <local-target> [--ref ...] [--token ...] [--proxy-base ...]`

Why:
- Matches the current script's mental model and avoids introducing a command hierarchy that the use case does not need.
- Keeps installation and release assets straightforward because every platform publishes one binary.

Alternatives considered:
- A multi-command CLI: rejected because there is only one core workflow today.
- A library crate plus thin CLI wrapper: rejected for the first iteration because the repository does not yet need a public API boundary.

### 2. Keep the download flow API-first, with proxy fallback only for anonymous requests

The Rust implementation will keep the current practical behavior:

- Use the GitHub Contents API to detect whether the target path is a file or directory.
- For files, prefer `download_url` when available, then fall back to `application/vnd.github.raw` through the Contents API.
- For directories, recursively enumerate entries and download each file using its relative path.
- Only use the public proxy fallback for anonymous requests or anonymous retry paths; never forward authentication tokens to the proxy.

Why:
- This preserves the current script's strongest behavior: reliable downloads with a safe anonymous fallback when GitHub rate limits or direct access is unstable.
- GitHub's Contents API already provides the metadata needed to distinguish files, directories, and unsupported entry types.

Alternatives considered:
- Use the Git Trees API for recursive enumeration: rejected for the first version because it complicates path handling and raw file download flow without a clear UX benefit here.
- Always use raw GitHub URLs: rejected because directory detection and authenticated private content are harder to handle consistently.

### 3. Stream file bytes directly to disk and prefer deterministic local path rules

All downloads will be streamed to disk rather than buffered in memory. Local destination behavior will mirror the current user-friendly rules:

- For a remote file, `local-target` may be a file path or an existing directory.
- For a remote directory, `local-target` is treated as the parent output directory unless it already ends with the same remote directory name.
- Parent directories are created automatically before writing files.

Why:
- Streaming is the safest default for unknown file sizes.
- Reusing the current path rules reduces user surprise during migration from the Python script.

Alternatives considered:
- Always write into a generated output folder: rejected because it would make single-file usage awkward.
- Buffer entire responses before writing: rejected due to unnecessary memory growth.

### 4. Model output as structured status events, not ad-hoc prints

The CLI will emit status lines through a small presentation layer that formats a handful of event types:

- startup summary
- directory scan summary
- per-file download line
- warning / fallback line
- final success summary
- final failure summary with suggestions

Color will be enabled by default on supported terminals, with a `--no-color` escape hatch for plain output and scripts.

Why:
- Separating output formatting from download logic makes the UX consistent and keeps future changes manageable.
- A small set of event types is enough to deliver “friendly” output without introducing a full TUI or noisy progress system.

Alternatives considered:
- Full progress bars for each file: rejected for the first release because they add noise and are less stable in redirected logs.
- Plain monochrome output only: rejected because the user explicitly wants color and scannability.

### 5. Convert low-level failures into categorized user guidance

The CLI will classify common failures before printing them:

- authentication or rate-limit issues (`401`, `403`, `429`)
- missing repo / ref / path (`404`)
- network connection or timeout failures
- local filesystem write failures
- unsupported remote entry types such as submodules or symlinks

Each category will produce a short explanation plus concrete suggestions, such as setting `GITHUB_TOKEN`, checking `--ref`, verifying the proxy URL, or inspecting local directory permissions.

Why:
- The primary product improvement over the script is not only speed or portability; it is clearer behavior when something goes wrong.
- Categorized errors also make automated tests easier because the expected UX is explicit.

Alternatives considered:
- Expose raw library errors directly: rejected because they are too low-level for a user-facing CLI.

### 6. Use a hand-written GitHub Actions release workflow triggered by version tags

Release automation will be implemented with repository-local workflows rather than introducing a meta release system. On pushes matching `v*`, GitHub Actions will:

- build release binaries for:
  - `x86_64-unknown-linux-musl`
  - `aarch64-unknown-linux-musl`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
- package artifacts as `.tar.gz` for Unix targets and `.zip` for Windows
- include the binary, `README.md`, and `LICENSE`
- generate a checksum manifest
- publish all assets to a GitHub Release associated with the pushed tag

Why:
- The repository is small and the release policy is simple, so a direct workflow is easier to audit and customize.
- This keeps naming, packaging layout, and future documentation under our control.

Alternatives considered:
- `cargo-dist`: rejected for now because it adds another layer of tooling before the project has proven it needs that complexity.
- Manual releases: rejected because the user explicitly wants tag-driven automation.

## Risks / Trade-offs

- [Public proxy availability can be inconsistent] -> Treat proxying as fallback only, emit explicit warnings when switching, and keep direct GitHub access as the primary path.
- [The first Rust release may not cover every edge case from the Python script] -> Preserve the same high-value behavior first and capture unsupported cases in specs and tests.
- [Cross-platform release packaging can fail due to target-specific tooling differences] -> Keep the first matrix narrow and use standard GitHub-hosted runners with explicit artifact naming.
- [Colored output may be undesirable in CI or redirected logs] -> Provide `--no-color` and avoid relying on color alone to communicate status.
- [Sequential directory downloads may be slower for large trees] -> Accept this trade-off for the first release in exchange for simpler logs, clearer failures, and lower implementation complexity.

## Migration Plan

1. Add the Rust crate and implement the CLI around the existing Python workflow semantics.
2. Add specs for the new capabilities so implementation and verification have stable requirements.
3. Implement the release workflow and asset packaging for tagged builds.
4. Update the README with installation, usage, and output examples.
5. Validate the Rust CLI locally, then publish the first tagged release.
6. Keep the Python script in the repository until the Rust CLI is verified, then decide whether it remains as reference code or is removed in a later change.

Rollback strategy:
- If the Rust CLI is not ready, avoid tagging a release and keep the repository on the Python script path.
- If a tagged release workflow produces broken assets, fix the workflow and publish a new version tag rather than mutating existing release artifacts.

## Open Questions

- Should the first release include a `--no-proxy` flag, or is `--proxy-base` plus an empty value sufficient?
- Do we want to keep the Python script as a supported fallback tool after the Rust CLI ships, or only as migration/reference material?
- Should future releases add package-manager-specific distribution channels, or is GitHub Release binary download enough for the first milestone?
