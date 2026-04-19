# gh-download

[简体中文](README.zh.md)

`gh-download` is a command-line tool for downloading a single file or an entire directory from a GitHub repository.

It works well when you want to:

- fetch one file without cloning a whole repository
- copy a directory without pulling full git history
- download content from public or private repositories in scripts or terminals

## Features

- Download a single file
- Download a directory recursively
- Choose a branch, tag, or commit with `--ref`
- Access private repositories with `GITHUB_TOKEN` or `GH_TOKEN`
- Fall back to a proxy when anonymous requests fail
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
gh-download <repo> <remote-path> <local-target> [--ref <ref>] [--token <token>] [--proxy-base <url>] [--lang <en|zh>] [--no-color]
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
- `--proxy-base`: Proxy prefix used when anonymous requests fail
- `--lang`: Explicit output language, `en` or `zh`
- `--no-color`: Disable ANSI color output

### Environment variables

- `GITHUB_TOKEN`: GitHub token, preferred over `GH_TOKEN`
- `GH_TOKEN`: Fallback GitHub token variable
- `GH_PROXY_BASE`: Default proxy prefix

### Language behavior

- English is the default output language
- If `LC_ALL`, `LC_MESSAGES`, or `LANG` resolves to a Chinese locale, output switches to Chinese automatically
- `--lang` has the highest priority and overrides locale detection

## Output examples

Success output:

```text
● gh-download
Repository owner/repo
Ref main
Remote src
Local /tmp/downloads

↻ Reading directory structure...
ℹ Found 3 files
↓ main.rs
↓ nested/lib.rs
↓ nested/mod.rs
✔ Done, saved to /tmp/downloads/src
Downloaded 3 files, skipped 0 entries
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
