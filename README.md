# gh-download

[简体中文](README.zh.md) | [Roadmap](ROADMAP.md)

`gh-download` is a command-line tool for downloading a single file or an entire directory from a GitHub repository.

It works well when you want to:

- fetch one file without cloning a whole repository
- copy a directory without pulling full git history
- download content from public or private repositories in scripts or terminals

## Features

- Download a single file
- Download a directory recursively
- Download directory files concurrently with `--concurrency` or `-c`
- Skip existing local files by default, with explicit `--overwrite` support
- Choose a branch, tag, or commit with `--ref`
- Access private repositories with `GITHUB_TOKEN` or `GH_TOKEN`
- Support explicit prefix-proxy modes for raw file downloads
- Support opt-in debug output for request URLs, token source, and strategy selection
- Show friendly output with actionable error suggestions
- Switch between English and Chinese automatically or explicitly

## Installation

### Install with Cargo

```bash
cargo install gh-download
```

### Download a prebuilt binary

Download the archive for your platform from the project's GitHub Releases page and extract it.

Available binaries cover:

- macOS Intel
- macOS Apple Silicon
- Linux x86_64
- Linux ARM64
- Windows x86_64

### Build from source

```bash
cargo build --release
```

The compiled binary will be available at:

```bash
./target/release/gh-download
```

## Usage

Basic syntax:

```bash
gh-download <repo> <remote-path> <local-target> [--ref <ref>] [--token <token>] [--proxy-base <url>] [--prefix-mode <direct|fallback|prefer>] [--concurrency <n>|-c <n>] [--overwrite] [--lang <en|zh>] [--debug] [--no-color]
```

Run `gh-download` without arguments to show the help screen in the effective language.

Download a single file:

```bash
gh-download openai/openai-python README.md ./README.md
```

Download a directory:

```bash
gh-download owner/repo src ./downloads
```

Download a directory from a specific branch:

```bash
gh-download owner/repo docs ./site-docs --ref main
```

Download a directory with higher concurrency:

```bash
gh-download owner/repo src ./downloads -c 8
```

Overwrite existing local files explicitly:

```bash
gh-download owner/repo src ./downloads --overwrite
```

Download from a private repository:

```bash
gh-download owner/private-repo docs ./docs --token "$GITHUB_TOKEN"
```

Force English output:

```bash
gh-download owner/repo docs ./docs --lang en
```

## Configuration

### Arguments

- `<repo>`: GitHub repository, such as `openai/openai-python`
- `<remote-path>`: Path inside the repository, such as `README.md` or `src/openai`
- `<local-target>`: Local output path
- `--ref`: Branch, tag, or commit SHA
- `--token`: GitHub token
- `--proxy-base`: URL-prefix proxy base used for anonymous raw file download retry or prefer mode
- `--prefix-mode`: Raw download prefix-proxy mode, `direct`, `fallback`, or `prefer`
- `--concurrency`, `-c`: Maximum number of concurrent file downloads for directory transfers. Must be at least `1`; defaults to `4`
- `--overwrite`: Replace existing local files instead of skipping them
- `--lang`: Explicit output language, `en` or `zh`
- `--debug`: Print debug diagnostics for request URLs, token source, and strategy selection
- `--no-color`: Disable ANSI color output

### Environment variables

- `GITHUB_TOKEN`: GitHub token, preferred over `GH_TOKEN`
- `GH_TOKEN`: Fallback GitHub token variable
- `GH_PROXY_BASE`: Explicit URL-prefix proxy base override
- `GH_DOWNLOAD_PREFIX_MODE`: Default raw download prefix-proxy mode
- `GH_DOWNLOAD_DEBUG`: Enable debug diagnostics when set to a truthy value

### Language behavior

- English is the default output language
- If `LC_ALL`, `LC_MESSAGES`, or `LANG` resolves to a Chinese locale, output switches to Chinese automatically
- `--lang` has the highest priority and overrides locale detection

### Prefix proxy behavior

- `--proxy-base` and `GH_PROXY_BASE` are used only for raw file download URLs, never for GitHub metadata API requests
- `--prefix-mode direct` is the default behavior
- `--prefix-mode fallback` retries a raw file download through the prefix proxy after a retryable direct-download failure, using the built-in `https://gh-proxy.com/` when no explicit proxy base is set
- `--prefix-mode prefer` tries the prefix proxy first and falls back to the direct raw file URL if the prefix attempt fails, using the built-in `https://gh-proxy.com/` when no explicit proxy base is set
- GitHub metadata API requests are not sent through URL-prefix fallback proxies such as `gh-proxy`
- When a token is present, `gh-download` will not forward that credential to the public fallback proxy
- When prefix retry is used, the warning output prints the full generated fallback URL with any embedded credentials redacted

### Directory download concurrency

- Directory downloads enumerate the remote tree first, then download files with up to `4` concurrent transfers by default
- Use `--concurrency <n>` or `-c <n>` to raise or lower the maximum number of in-flight file downloads for a directory transfer
- Use `--concurrency 1` or `-c 1` if you want an explicit sequential mode for troubleshooting or low-resource environments
- Single-file downloads accept `--concurrency` and `-c`, but still download only one resolved file target
- Concurrent directory downloads preserve the same relative-path layout on disk; only the order of progress lines may vary

### Local write behavior

- Existing local files are skipped by default instead of being overwritten implicitly
- Pass `--overwrite` if you want the CLI to replace existing local files
- The same rule applies to direct file downloads and to per-file writes inside directory downloads
- Skip decisions are based on whether the resolved local file target already exists; the CLI does not compare file contents in this mode

### Debug behavior

- `--debug` and `GH_DOWNLOAD_DEBUG` enable flow-level diagnostics
- Debug output includes the generated GitHub metadata URL, the detected token source label, the resolved raw download URL, the generated prefix URL when applicable, and the selected raw download strategy
- Debug output is written to `stderr` and does not change download behavior

Recommended setup:

- Keep the default `direct` mode for portability in open source usage
- Set `GH_DOWNLOAD_PREFIX_MODE=fallback` if you want raw file URLs retried through the built-in gh-proxy after direct-download failures
- Set `GH_DOWNLOAD_PREFIX_MODE=prefer` if you want raw file URLs to try the built-in gh-proxy before direct download
- Set `GH_PROXY_BASE=...` only when you want to override the built-in prefix proxy

## Output examples

Success output:

```text
-------------------------------------
📦 Repository:owner/repo
🌿 Ref:main
📂 Remote Path:src
💾 Local Path:/tmp/downloads
-------------------------------------
🔎 Found 3 files in directory: src
📁 Created Local Directory:/tmp/downloads/src
-------------------------------------
⬇️ Download:main.rs
⬇️ Download:nested/lib.rs
⬇️ Download:nested/mod.rs
-------------------------------------
✅ Done: owner/repo src saved to /tmp/downloads/src
Downloaded 3 files, skipped 0 existing files, skipped 0 unsupported entries
```

Skip-existing output:

```text
-------------------------------------
📦 Repository:owner/repo
🌿 Ref:default branch
📂 Remote Path:README.md
💾 Local Path:/tmp/README.md
-------------------------------------
⏭ Skipping existing file: README.md
-------------------------------------
✅ Done: owner/repo README.md saved to /tmp/README.md
Downloaded 0 files, skipped 1 existing files, skipped 0 unsupported entries
```

Prefix-proxy output:

```text
-------------------------------------
📦 Repository:owner/repo
🌿 Ref:default branch
📂 Remote Path:README.md
💾 Local Path:/tmp/README.md
-------------------------------------
⚠ Direct file download failed, retrying through prefix proxy: https://gh-proxy.com/https://raw.githubusercontent.com/OWNER/REPO/REF/README.md
⬇️ Download:README.md
-------------------------------------
✅ Done: owner/repo README.md saved to /tmp/README.md
```

Debug output:

```text
[debug] metadata-url: https://api.github.com/repos/owner/repo/contents/README.md
[debug] token-source: GITHUB_TOKEN
[debug] download-url: https://raw.githubusercontent.com/owner/repo/main/README.md
[debug] prefix-url: https://gh-proxy.com/https://raw.githubusercontent.com/owner/repo/main/README.md
[debug] raw-download-strategy: prefix-proxy
```

Error output:

```text
✖ Download failed
Reason: GitHub authentication failed or the rate limit was hit (HTTP 403)
Suggestions:
- Set GITHUB_TOKEN or GH_TOKEN in the environment
- Or rerun with --token <token>
- If direct GitHub access is unstable, verify that --proxy-base is reachable
```

## Contributing

Issues and pull requests are welcome.

Common local commands:

```bash
cargo fmt
cargo test
```

If you change CLI behavior, especially user-facing output, arguments, or download rules, update the matching specs under `openspec/` as well.

## License

This project is licensed under the [MIT License](LICENSE).
