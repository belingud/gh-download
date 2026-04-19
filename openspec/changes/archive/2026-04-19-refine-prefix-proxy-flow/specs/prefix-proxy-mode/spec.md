## ADDED Requirements

### Requirement: Raw file downloads support explicit prefix proxy modes
The CLI SHALL support an explicit prefix proxy mode for raw file downloads. The mode SHALL be configurable by CLI argument and environment variable, and it MUST support `direct`, `fallback`, and `prefer`. The default mode SHALL be `direct`.

#### Scenario: Default mode uses direct raw download
- **WHEN** a user runs the CLI without setting the prefix proxy mode
- **THEN** raw file downloads are attempted directly without prefix-proxy rewriting

#### Scenario: Fallback mode retries raw file download through prefix proxy
- **WHEN** the prefix proxy mode is `fallback` and an anonymous direct raw file download fails with a retryable network or HTTP error
- **THEN** the CLI retries the same raw file download through the configured URL-prefix proxy

#### Scenario: Prefer mode uses prefix proxy first
- **WHEN** the prefix proxy mode is `prefer` and a prefix proxy base is configured
- **THEN** the CLI first attempts the raw file download through the configured URL-prefix proxy before trying the direct raw file URL

#### Scenario: Prefer mode without proxy base uses built-in gh-proxy
- **WHEN** the prefix proxy mode is `prefer` and no prefix proxy base is configured
- **THEN** the CLI first attempts the raw file download through the built-in `https://gh-proxy.com/` prefix proxy before trying the direct raw file URL
