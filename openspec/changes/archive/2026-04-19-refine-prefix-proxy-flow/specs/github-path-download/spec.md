## MODIFIED Requirements

### Requirement: CLI retries anonymous raw file downloads through the configured URL-prefix proxy when direct access fails
The CLI SHALL ignore ambient system proxy environment variables for its direct HTTP(S) requests. GitHub metadata API requests MUST NOT be retried through URL-prefix fallback proxies such as `gh-proxy`. When the CLI is running without an authentication token, the prefix proxy mode is `fallback`, and a prefix proxy base is configured, it SHALL retry eligible raw file download requests through the configured URL-prefix proxy after direct GitHub rate-limit, server-side, or network failures. The CLI MUST NOT forward authentication credentials to the public proxy path.

#### Scenario: Direct request ignores system proxy configuration
- **WHEN** a user configures a standard proxy environment variable such as `HTTP_PROXY`, `HTTPS_PROXY`, or `ALL_PROXY`
- **THEN** the CLI still sends direct requests without using that ambient system proxy configuration

#### Scenario: Anonymous GitHub metadata API request is rate limited
- **WHEN** a direct anonymous GitHub metadata API request fails with a retryable HTTP status such as `403`, `429`, or a transient server error
- **THEN** the CLI does not retry that metadata request through the URL-prefix fallback proxy

#### Scenario: Anonymous raw file download request is rate limited
- **WHEN** a direct anonymous raw file download request fails with a retryable HTTP status such as `403`, `429`, or a transient server error
- **THEN** the CLI retries the raw file URL through the configured URL-prefix proxy and informs the user that proxy fallback is being used

#### Scenario: Authenticated request fails
- **WHEN** a request is made with an explicit token or an environment token
- **THEN** the CLI does not send that credential through the proxy fallback path

### Requirement: CLI provides concise colored status output and actionable failure guidance
The CLI SHALL print a startup summary that includes the repository, ref selection, remote path, and local target. It SHALL print concise progress messages during downloads and a completion summary at the end of a successful operation. On failure, the CLI MUST present a short explanation plus at least one remediation suggestion for common categories including authentication, missing path or ref, network failure, and local filesystem write failure.

#### Scenario: Successful download reports progress and completion
- **WHEN** a download completes successfully
- **THEN** the CLI shows a readable progress trail and a final success summary that identifies the saved local path

#### Scenario: Prefix fallback proxy retry is used
- **WHEN** the CLI retries an eligible anonymous raw file download through `--proxy-base`
- **THEN** the warning output identifies the full generated fallback URL and redacts any embedded credentials before printing it

#### Scenario: Direct raw file download fails before Raw API fallback
- **WHEN** a direct file `download_url` attempt fails and the CLI continues to the next raw download strategy
- **THEN** the CLI prints a short warning that names the failure stage without requiring the user to infer what "direct URL unavailable" means

#### Scenario: Download fails due to missing authentication
- **WHEN** GitHub rejects a request because authentication is missing or anonymous access is rate limited
- **THEN** the CLI reports the failure in user-facing terms and suggests providing `--token` or setting `GITHUB_TOKEN` or `GH_TOKEN`
