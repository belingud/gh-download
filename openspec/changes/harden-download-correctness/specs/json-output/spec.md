## MODIFIED Requirements

### Requirement: CLI supports explicit machine-readable JSON output
The CLI SHALL support an explicit `--json` flag that switches the command output to a machine-readable JSON result. When `--json` is enabled, the CLI SHALL emit one final JSON document on stdout instead of the default human-readable progress and summary text. This requirement SHALL also apply to argument-validation and CLI usage failures: when `--json` is present in the arguments but the invocation is rejected during argument parsing, the CLI MUST emit one JSON failure document on stdout and MUST NOT emit human-readable error text on stdout, while keeping the non-zero usage exit status.

#### Scenario: Successful download uses JSON output
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --json` and the download succeeds
- **THEN** the CLI writes one JSON success document to stdout instead of human-readable startup, progress, and completion text

#### Scenario: Failed download uses JSON output
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --json` and the download fails
- **THEN** the CLI writes one JSON failure document to stdout instead of the default human-readable error output

#### Scenario: Invalid arguments use JSON output
- **WHEN** a user runs `gh-download --json` without the required positional arguments, or passes an invalid option value with `--json`
- **THEN** the CLI writes one JSON failure document with the usage error information to stdout and exits with the usage-error status instead of printing only human-readable text

### Requirement: JSON success output includes resolved path and download statistics
The JSON success document SHALL include a machine-readable success indicator, the resolved saved path, the resolved commit SHA of the downloaded content, and aggregate download statistics needed by automation. Those statistics MUST distinguish downloaded files, skipped existing files, skipped unsupported entries, skipped symlinks, and skipped submodules.

#### Scenario: Directory download success reports aggregate counts
- **WHEN** a directory download completes successfully in JSON mode
- **THEN** the JSON success document includes the saved path, the resolved commit SHA, and counts for downloaded files, skipped existing files, skipped unsupported entries, skipped symlinks, and skipped submodules

#### Scenario: Single file download success reports resolved commit
- **WHEN** a single file download completes successfully in JSON mode
- **THEN** the JSON success document includes the resolved commit SHA used for that download

### Requirement: JSON failure output includes classified error information
The JSON failure document SHALL include a machine-readable success indicator plus the classified failure title, a stable language-independent error code, the reason, and suggestions derived from the CLI's existing error classification flow. The code MUST be stable across releases for the same failure category and MUST be independent of the effective output language. When the failure happens during a directory transfer, the JSON failure document MUST also include the partial download statistics and a list of failed entries, where each failed entry identifies the remote path, the resolved local path, and the failure reason. JSON field names MUST remain stable regardless of the effective output language.

#### Scenario: Authentication failure in JSON mode
- **WHEN** GitHub rejects a JSON-mode download because authentication is missing or rate limited
- **THEN** the JSON failure document includes structured fields for the failure code, title, reason, and suggestions

#### Scenario: Usage failure in JSON mode
- **WHEN** a user passes invalid arguments together with `--json`
- **THEN** the JSON failure document includes a usage error code and the argument-validation message

#### Scenario: Partial directory failure in JSON mode
- **WHEN** a directory download saves some files and then fails without `--keep-going`
- **THEN** the JSON failure document includes the downloaded and skipped counts plus the failed entry that stopped the transfer

#### Scenario: Keep-going failures in JSON mode
- **WHEN** a directory download runs with `--keep-going` and several files fail
- **THEN** the JSON failure document includes the aggregate statistics and one failed entry for every file that failed
