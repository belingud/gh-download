## ADDED Requirements

### Requirement: CLI supports explicit machine-readable JSON output
The CLI SHALL support an explicit `--json` flag that switches the command output to a machine-readable JSON result. When `--json` is enabled, the CLI SHALL emit one final JSON document on stdout instead of the default human-readable progress and summary text.

#### Scenario: Successful download uses JSON output
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --json` and the download succeeds
- **THEN** the CLI writes one JSON success document to stdout instead of human-readable startup, progress, and completion text

#### Scenario: Failed download uses JSON output
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --json` and the download fails
- **THEN** the CLI writes one JSON failure document to stdout instead of the default human-readable error output

### Requirement: JSON success output includes resolved path and download statistics
The JSON success document SHALL include a machine-readable success indicator, the resolved saved path, and aggregate download statistics needed by automation. Those statistics MUST distinguish downloaded files, skipped existing files, and skipped unsupported entries.

#### Scenario: Directory download success reports aggregate counts
- **WHEN** a directory download completes successfully in JSON mode
- **THEN** the JSON success document includes the saved path plus counts for downloaded files, skipped existing files, and skipped unsupported entries

### Requirement: JSON failure output includes classified error information
The JSON failure document SHALL include a machine-readable success indicator plus the classified failure title, reason, and suggestions derived from the CLI's existing error classification flow. JSON field names MUST remain stable regardless of the effective output language.

#### Scenario: Authentication failure in JSON mode
- **WHEN** GitHub rejects a JSON-mode download because authentication is missing or rate limited
- **THEN** the JSON failure document includes structured fields for the failure title, reason, and suggestions
