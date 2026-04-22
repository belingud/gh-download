---
name: gh-download
description: Use this skill when you need to download a single file or a whole directory from a GitHub repository path with the gh-download CLI, including installation checks, config-file defaults, private-repo token usage, GitHub Enterprise API base support, prefix-proxy modes, and machine-readable JSON output.
version: 0.4.0
metadata:
  openclaw:
    homepage: https://github.com/belingud/gh-download
    requires:
      anyBins:
        - gh-download
---

# gh-download

## What This Tool Does

`gh-download` is a command-line tool for downloading a single file or an entire directory from a GitHub repository path without cloning the whole repository.

Typical use cases:

- Download a single file.
- Download a directory while preserving its structure.
- Automate repository path downloads in scripts, terminals, or other AI agent workflows.
- Access a specific path inside a private repository.
- Return machine-readable JSON output for downstream processing.

## Notes

- For single-file downloads, the local target can be either a file path or an existing directory such as `.`.
- For directory downloads, the remote relative path structure is preserved. In most cases, the tool creates a same-named directory under the local target unless the local target already ends with that directory name.
- Existing local files are not overwritten by default. Use `--overwrite` explicitly if replacement is intended.
- Raw file downloads can use prefix-proxy modes, but GitHub metadata API requests must not go through URL-prefix proxies such as `gh-proxy`.
- When a token is present, authentication credentials are not forwarded to a public prefix proxy.
- `--json` switches `stdout` to a single final JSON document. If `--debug` is also enabled, debug output still goes to `stderr`.
- The CLI can read persisted defaults from `~/.config/gh-download/config.toml`.
- `--config <path>` makes the CLI read only that TOML config file for the current invocation.
- Supported config keys are `token`, `api_base`, `proxy_base`, `prefix_mode`, `concurrency`, and `lang`.
- Effective precedence is: CLI argument, then config file, then environment variable, then built-in default.

## Make Sure It Is Installed First

First check whether it is already installed:

```bash
gh-download --version
```

If it is not installed yet, use one of the following options.

Install with Cargo:

```bash
cargo install gh-download
```

Build from source inside this repository:

```bash
cargo build --release
```

The compiled binary will be located at:

```bash
./target/release/gh-download
```

If you only need to run it temporarily inside this repository:

```bash
cargo run -- <repo> <remote-path> <local-target> [options]
```

## Basic Usage

Basic syntax:

```bash
gh-download <repo> <remote-path> <local-target> [options]
```

Positional arguments:

- `<repo>`: GitHub repository in `owner/repo` format
- `<remote-path>`: Path inside the repository, such as `README.md`, `src`, or `docs/api`
- `<local-target>`: Local output path

## Parameters

- `--ref <ref>`: Specify a branch, tag, or commit SHA
- `--config <path>`: Read options from this TOML config file only. When omitted, `~/.config/gh-download/config.toml` is used if it exists
- `--token <token>`: Specify a GitHub token. If omitted, the CLI reads `GITHUB_TOKEN` or `GH_TOKEN`
- `--api-base <url>`: Specify a custom GitHub metadata API base URL for GitHub Enterprise or compatible deployments
- `--proxy-base <url>`: Specify the URL-prefix proxy base used for raw file downloads
- `--prefix-mode <direct|fallback|prefer>`: Specify the raw download prefix-proxy mode. The default is `direct`
- `--concurrency <n>` or `-c <n>`: Specify the maximum number of concurrent file downloads for directory transfers. Minimum `1`, default `4`
- `--overwrite`: Overwrite existing local files
- `--json`: Emit the final result as JSON
- `--lang <en|zh>`: Explicitly set the output language
- `--debug`: Print request URLs, token source, and download strategy for debugging
- `--no-color`: Disable ANSI color output

Related environment variables:

- `GITHUB_TOKEN`: Preferred GitHub token when neither CLI nor config file provides `token`
- `GH_TOKEN`: Fallback GitHub token
- `GH_PROXY_BASE`: Default prefix-proxy base when neither CLI nor config file provides `proxy_base`
- `GH_DOWNLOAD_PREFIX_MODE`: Default prefix-proxy mode when neither CLI nor config file provides `prefix_mode`
- `GH_DOWNLOAD_DEBUG`: Enable debug output when set to a truthy value

Config file example:

```toml
api_base = "https://api.github.com"
proxy_base = "https://gh-proxy.com/"
prefix_mode = "direct"
concurrency = 4
lang = "zh"
token = "xxxx"
```

## Examples

Download a single file:

```bash
gh-download openai/openai-python README.md .
```

If the local target is an existing directory such as `.`, the CLI saves the file under its remote file name automatically.

Download an entire directory:

```bash
gh-download owner/repo src ./downloads
```

Download a directory from a specific branch:

```bash
gh-download owner/repo docs ./site-docs --ref main
```

Increase directory download concurrency:

```bash
gh-download owner/repo src ./downloads -c 8
```

Explicitly overwrite existing local files:

```bash
gh-download owner/repo src ./downloads --overwrite
```

Download content from a private repository:

```bash
gh-download owner/private-repo docs ./docs --token "$GITHUB_TOKEN"
```

Use a custom GitHub metadata API base:

```bash
gh-download owner/repo docs ./docs --api-base https://ghe.example.com/api/v3
```

Use an explicit config file:

```bash
gh-download owner/repo docs ./docs --config ./gh-download.toml
```

Use anonymous raw download fallback proxy mode:

```bash
gh-download owner/repo src ./downloads --prefix-mode fallback
```

Run directly from this repository:

```bash
cargo run -- owner/repo src ./downloads --json --no-color
```

## JSON Output

Enable it with:

```bash
gh-download owner/repo README.md . --json
```

Success output includes:

- `success`
- `saved_path`
- `stats.files_downloaded`
- `stats.skipped_existing_files`
- `stats.skipped_unsupported_entries`

Failure output includes:

- `success`
- `error.title`
- `error.reason`
- `error.suggestions`

Typical success payload:

```json
{
  "success": true,
  "saved_path": "/tmp/downloads/src",
  "stats": {
    "files_downloaded": 2,
    "skipped_existing_files": 1,
    "skipped_unsupported_entries": 0
  }
}
```

Typical failure payload:

```json
{
  "success": false,
  "error": {
    "title": "Download failed",
    "reason": "authentication is missing or rate limited",
    "suggestions": [
      "provide --token",
      "set GITHUB_TOKEN or GH_TOKEN"
    ]
  }
}
```

For stable automation, prefer `--json` and parse only the JSON fields instead of relying on human-readable progress text.
