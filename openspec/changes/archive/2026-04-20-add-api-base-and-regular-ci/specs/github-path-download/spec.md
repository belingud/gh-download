## MODIFIED Requirements

### Requirement: CLI accepts GitHub path download inputs
The `gh-download` CLI SHALL accept a repository identifier, a repository-relative remote path, and a local target path as positional arguments when a download invocation is provided. When the CLI is invoked without any user-provided arguments, it SHALL print the localized help text and exit successfully instead of reporting missing required positional arguments. The CLI SHALL also support `--ref`, `--token`, `--api-base`, `--proxy-base`, `--prefix-mode`, `--concurrency`, `-c`, `--overwrite`, `--json`, `--debug`, and `--lang` options, and it MUST use `GITHUB_TOKEN` or `GH_TOKEN` as the default token source when `--token` is not provided.

#### Scenario: User provides explicit CLI arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --ref main --token abc --api-base https://ghe.example.com/api/v3 --proxy-base https://gh-proxy.com/ --lang en`
- **THEN** the CLI accepts the invocation and uses the provided repository, remote path, local target, ref, token, API base, proxy base, and language for the download operation

#### Scenario: User provides explicit machine-readable output and debug arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --json --debug`
- **THEN** the CLI accepts the invocation and uses JSON result output and debug diagnostics for the download operation

#### Scenario: User relies on environment token defaults
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md` with `GITHUB_TOKEN` or `GH_TOKEN` set
- **THEN** the CLI uses the environment token automatically without requiring `--token`

#### Scenario: User runs the CLI without arguments
- **WHEN** a user runs `gh-download` without any additional arguments
- **THEN** the CLI prints the help output for the effective language and exits successfully without a missing-arguments error

### Requirement: CLI detects file versus directory targets from GitHub metadata
The CLI SHALL query GitHub metadata for the requested remote path before downloading content. It MUST distinguish between file and directory targets and it MUST fail with a user-visible error when GitHub returns an unsupported or unexpected target type. When `--api-base` is provided, the metadata request MUST use that explicit API base instead of the default public GitHub API base.

#### Scenario: Remote path resolves to a file
- **WHEN** the requested remote path points to a file in the repository
- **THEN** the CLI treats the request as a file download and resolves a single local output target

#### Scenario: Remote path resolves to a directory
- **WHEN** the requested remote path points to a directory in the repository
- **THEN** the CLI treats the request as a directory download and prepares to enumerate its contents recursively

#### Scenario: Custom API base is used for metadata detection
- **WHEN** a user runs `gh-download owner/repo docs ./docs --api-base https://ghe.example.com/api/v3`
- **THEN** the CLI sends its GitHub contents metadata requests to `https://ghe.example.com/api/v3` instead of `https://api.github.com`

### Requirement: CLI enforces raw-download proxy boundaries
The CLI SHALL ignore ambient system proxy environment variables for its direct HTTP(S) requests. GitHub metadata API requests MUST NOT be retried through URL-prefix proxies such as `gh-proxy`, including when `--api-base` is set to a custom GitHub-compatible API endpoint. Anonymous raw file downloads SHALL follow the explicit prefix-proxy mode behavior defined by the `prefix-proxy-mode` capability. The CLI MUST NOT forward authentication credentials to the public proxy path.

#### Scenario: Direct request ignores system proxy configuration
- **WHEN** a user configures a standard proxy environment variable such as `HTTP_PROXY`, `HTTPS_PROXY`, or `ALL_PROXY`
- **THEN** the CLI still sends direct requests without using that ambient system proxy configuration

#### Scenario: Anonymous GitHub metadata API request is rate limited
- **WHEN** a direct anonymous GitHub metadata API request fails with a retryable HTTP status such as `403`, `429`, or a transient server error
- **THEN** the CLI does not retry that metadata request through the URL-prefix fallback proxy

#### Scenario: Anonymous raw file download follows prefix mode behavior
- **WHEN** an anonymous raw file download is attempted
- **THEN** the CLI applies the configured prefix-proxy mode behavior for the raw file URL without changing the metadata API path

#### Scenario: Authenticated request fails
- **WHEN** a request is made with an explicit token or an environment token
- **THEN** the CLI does not send that credential through the proxy fallback path
