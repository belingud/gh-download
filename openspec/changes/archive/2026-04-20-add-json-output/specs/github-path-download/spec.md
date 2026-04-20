## MODIFIED Requirements

### Requirement: CLI accepts GitHub path download inputs
The `gh-download` CLI SHALL accept a repository identifier, a repository-relative remote path, and a local target path as positional arguments when a download invocation is provided. When the CLI is invoked without any user-provided arguments, it SHALL print the localized help text and exit successfully instead of reporting missing required positional arguments. The CLI SHALL also support `--ref`, `--token`, `--proxy-base`, `--prefix-mode`, `--concurrency`, `-c`, `--overwrite`, `--json`, `--debug`, and `--lang` options, and it MUST use `GITHUB_TOKEN` or `GH_TOKEN` as the default token source when `--token` is not provided.

#### Scenario: User provides explicit CLI arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --ref main --token abc --proxy-base https://gh-proxy.com/ --lang en`
- **THEN** the CLI accepts the invocation and uses the provided repository, remote path, local target, ref, token, proxy base, and language for the download operation

#### Scenario: User provides explicit machine-readable output and debug arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --json --debug`
- **THEN** the CLI accepts the invocation and uses JSON result output and debug diagnostics for the download operation

#### Scenario: User relies on environment token defaults
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md` with `GITHUB_TOKEN` or `GH_TOKEN` set
- **THEN** the CLI uses the environment token automatically without requiring `--token`

#### Scenario: User runs the CLI without arguments
- **WHEN** a user runs `gh-download` without any additional arguments
- **THEN** the CLI prints the help output for the effective language and exits successfully without a missing-arguments error

### Requirement: CLI provides concise colored status output and actionable failure guidance
The CLI SHALL print a structured startup summary with separators that includes the repository, ref selection, remote path, and local target. For directory downloads, it SHALL print the discovered file count with the remote directory and the created local directory before file progress. It SHALL print concise per-file download progress messages, including when an existing local file is skipped, and a structured completion summary at the end of a successful operation. On failure, the CLI MUST present a short explanation plus at least one remediation suggestion for common categories including authentication, missing path or ref, network failure, and local filesystem write failure. When JSON output mode is enabled, these human-readable progress, completion, and error messages SHALL be suppressed from stdout in favor of the machine-readable JSON result.

#### Scenario: Successful download reports progress and completion
- **WHEN** a download completes successfully without JSON mode
- **THEN** the CLI shows a readable progress trail and a final success summary that identifies the saved local path

#### Scenario: Existing local file is skipped
- **WHEN** the CLI skips writing a resolved local file because it already exists and overwrite mode is not enabled
- **THEN** the progress output identifies that file as skipped rather than downloaded

#### Scenario: JSON mode suppresses human-readable stdout output
- **WHEN** a user enables JSON output mode for a download
- **THEN** the CLI does not print the default human-readable startup, progress, completion, or error text to stdout

#### Scenario: Anonymous fallback proxy retry is used
- **WHEN** the CLI retries an eligible anonymous raw file download through `--proxy-base`
- **THEN** the warning output identifies the full generated fallback URL and redacts any embedded credentials before printing it

#### Scenario: Direct raw file download fails before Raw API fallback
- **WHEN** a direct file `download_url` attempt fails and the CLI continues to the next raw download strategy
- **THEN** the CLI prints a short warning that names the failure stage without requiring the user to infer what "direct URL unavailable" means

#### Scenario: Download fails due to missing authentication
- **WHEN** GitHub rejects a request because authentication is missing or anonymous access is rate limited
- **THEN** the CLI reports the failure in user-facing terms and suggests providing `--token` or setting `GITHUB_TOKEN` or `GH_TOKEN`
