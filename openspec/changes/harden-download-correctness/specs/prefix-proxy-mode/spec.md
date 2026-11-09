## MODIFIED Requirements

### Requirement: Raw file downloads support explicit prefix proxy modes
The CLI SHALL support an explicit prefix proxy mode for raw file downloads. The mode SHALL be configurable by CLI argument, configuration file, and environment variable, and it MUST support `direct`, `fallback`, and `prefer`. The precedence SHALL be: explicit CLI argument, then configuration file, then environment variable, then the default `direct` mode.

The effective raw download strategy MUST be token-aware. When an effective token is available and the target repository is private, the CLI SHALL download file content through the token-authenticated GitHub Contents raw media type instead of first attempting an anonymous raw file URL, and it MUST NOT print a per-file anonymous raw failure warning. When the target repository is public, or when no token is available, the CLI SHALL use the raw file URL strategy with the configured prefix proxy mode. Prefix-proxy retries SHALL apply to anonymous raw URL attempts even when a token is configured, because those attempts carry no credential, and the CLI MUST NOT forward that token to the proxy.

When the effective `api_base` is a custom GitHub Enterprise-compatible base, the CLI SHALL download file content through the token-authenticated Contents raw media type, and prefix proxy modes MUST NOT be applied to those raw-media requests or to metadata requests.

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
- **WHEN** the prefix proxy mode is `prefer` and the effective raw strategy is the anonymous raw file URL
- **THEN** the CLI first attempts the raw file download through the effective URL-prefix proxy before trying the direct raw file URL

#### Scenario: Prefer mode without proxy base uses built-in gh-proxy
- **WHEN** the prefix proxy mode is `prefer` and no prefix proxy base is configured
- **THEN** the CLI first attempts the raw file download through the built-in `https://gh-proxy.com/` prefix proxy before trying the direct raw file URL

#### Scenario: Private repository uses the token-authenticated raw media type
- **WHEN** a token is configured, the target repository is private, and prefix proxy mode is `prefer` or `fallback`
- **THEN** the CLI downloads file content through the token-authenticated Contents raw media type without sending the private raw file URL to the prefix proxy and without per-file anonymous raw failure warnings

#### Scenario: Configured token does not disable anonymous proxy retry
- **WHEN** a token is configured, the target repository is public, prefix proxy mode is `fallback`, and an anonymous raw file URL attempt fails with a retryable error
- **THEN** the CLI retries that raw file URL through the effective prefix proxy without forwarding the token

#### Scenario: Custom API base uses raw media type
- **WHEN** the effective `api_base` is a custom GitHub Enterprise-compatible base
- **THEN** the CLI downloads file content through the token-authenticated Contents raw media type for that API base, and metadata requests and raw-media requests are not routed through prefix proxies
