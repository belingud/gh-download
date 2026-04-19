## 1. Project Setup

- [x] 1.1 Create the Rust CLI crate structure for `gh-download` and add the core dependencies for argument parsing, HTTP requests, serialization, colored output, and error handling
- [x] 1.2 Define the CLI interface with positional arguments plus `--ref`, `--token`, `--proxy-base`, and `--no-color`, including environment-token defaults and help text

## 2. Download Engine

- [x] 2.1 Implement GitHub metadata requests that normalize repository paths and detect whether the requested remote path is a file or directory
- [x] 2.2 Implement file download streaming with support for `download_url` first and raw Contents API fallback when needed
- [x] 2.3 Implement recursive directory enumeration, relative path mapping, parent directory creation, and deterministic local target resolution
- [x] 2.4 Implement anonymous proxy fallback logic for retryable GitHub failures without forwarding authentication credentials

## 3. User Experience and Error Handling

- [x] 3.1 Add a presentation layer for startup summaries, directory scan summaries, per-file progress lines, warnings, and final success output with optional color disabling
- [x] 3.2 Categorize common failures into user-facing error messages with remediation suggestions for authentication, missing path or ref, network failures, local write failures, and unsupported entry types
- [x] 3.3 Add automated tests for argument handling, path resolution, error classification, and representative file or directory download flows
- [x] 3.4 Add locale-aware language selection so CLI output defaults to English and switches to Chinese when locale settings indicate Chinese, with an explicit `--lang` override

## 4. Release Automation and Documentation

- [x] 4.1 Create a GitHub Actions workflow that builds release binaries for the supported Linux, macOS, and Windows targets when a `v*` tag is pushed
- [x] 4.2 Package release artifacts with the executable, `README.md`, and `LICENSE`, generate checksums, and publish them to the matching GitHub Release
- [x] 4.3 Update the README with installation guidance, CLI usage, output examples, and release behavior, then run local validation for build and test commands
