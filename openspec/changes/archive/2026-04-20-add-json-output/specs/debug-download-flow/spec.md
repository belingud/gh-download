## MODIFIED Requirements

### Requirement: CLI supports explicit debug output for download flow diagnostics
The CLI SHALL support an explicit debug mode that prints detailed download-flow diagnostics only when enabled by CLI argument or environment variable. The debug output MUST be disabled by default and MUST NOT alter the normal download strategy on its own. When JSON output mode is enabled at the same time, debug diagnostics SHALL continue to be emitted on stderr so the JSON payload on stdout remains machine-readable.

#### Scenario: Debug mode is disabled by default
- **WHEN** a user runs the CLI without enabling debug mode
- **THEN** the CLI does not print diagnostic request-construction or strategy-selection output

#### Scenario: Debug mode prints request URLs, token source, and strategy choices
- **WHEN** a user enables debug mode during a download
- **THEN** the CLI prints the generated GitHub metadata URL, the detected token source label when one is recognized, the resolved raw file download URL, the generated prefix-proxy URL when applicable, and the selected raw download strategy

#### Scenario: Debug mode does not change download behavior
- **WHEN** a user enables debug mode
- **THEN** the CLI performs the same download decisions it would have made without debug mode, aside from printing diagnostic information

#### Scenario: Debug mode stays on stderr in JSON mode
- **WHEN** a user enables both JSON output mode and debug mode
- **THEN** the CLI keeps debug diagnostics on stderr instead of mixing them into the stdout JSON result
