## MODIFIED Requirements

### Requirement: Raw file downloads support explicit prefix proxy modes
The CLI SHALL support an explicit prefix proxy mode for raw file downloads. The mode SHALL be configurable by CLI argument, configuration file, and environment variable, and it MUST support `direct`, `fallback`, and `prefer`. The precedence SHALL be: explicit CLI argument, then configuration file, then environment variable, then the default `direct` mode.

#### Scenario: Default mode uses direct raw download
- **WHEN** a user runs the CLI without setting the prefix proxy mode in CLI arguments, configuration file, or environment variables
- **THEN** raw file downloads are attempted directly without prefix-proxy rewriting

#### Scenario: Config file overrides environment prefix mode
- **WHEN** the configuration file contains `prefix_mode = "prefer"`, `GH_DOWNLOAD_PREFIX_MODE` is set to `fallback`, and the user does not provide `--prefix-mode`
- **THEN** the CLI uses `prefer` as the effective prefix proxy mode

#### Scenario: Fallback mode retries raw file download through prefix proxy
- **WHEN** the prefix proxy mode is `fallback` and an anonymous direct raw file download fails with a retryable network or HTTP error
- **THEN** the CLI retries the same raw file download through the effective URL-prefix proxy

#### Scenario: Fallback mode without proxy base uses built-in gh-proxy
- **WHEN** the prefix proxy mode is `fallback` and no prefix proxy base is configured
- **THEN** the CLI uses the built-in `https://gh-proxy.com/` prefix proxy for the retry attempt

#### Scenario: Prefer mode uses prefix proxy first
- **WHEN** the prefix proxy mode is `prefer`
- **THEN** the CLI first attempts the raw file download through the effective URL-prefix proxy before trying the direct raw file URL

#### Scenario: Prefer mode without proxy base uses built-in gh-proxy
- **WHEN** the prefix proxy mode is `prefer` and no prefix proxy base is configured
- **THEN** the CLI first attempts the raw file download through the built-in `https://gh-proxy.com/` prefix proxy before trying the direct raw file URL
