## MODIFIED Requirements

### Requirement: CLI accepts GitHub path download inputs
The `gh-download` CLI SHALL accept a repository identifier, a repository-relative remote path, and a local target path as positional arguments when a download invocation is provided. When the CLI is invoked without any user-provided arguments, it SHALL print the localized help text and exit successfully instead of reporting missing required positional arguments. The CLI SHALL also support `--ref`, `--token`, `--api-base`, `--proxy-base`, `--prefix-mode`, `--concurrency`, `-c`, `--overwrite`, `--json`, `--debug`, `--lang`, and `--config` options. When `--token` is not provided, the CLI MUST resolve the effective token from the configuration file before falling back to `GITHUB_TOKEN` or `GH_TOKEN`.

#### Scenario: User provides explicit CLI arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --ref main --token abc --api-base https://ghe.example.com/api/v3 --proxy-base https://gh-proxy.com/ --lang en`
- **THEN** the CLI accepts the invocation and uses the provided repository, remote path, local target, ref, token, API base, proxy base, and language for the download operation

#### Scenario: User relies on configuration file defaults
- **WHEN** a user runs `gh-download owner/repo src ./downloads` and the active configuration file contains `api_base`, `token`, and `lang`
- **THEN** the CLI uses those configuration-file values for the invocation without requiring the matching CLI options

#### Scenario: User provides explicit machine-readable output and debug arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --json --debug`
- **THEN** the CLI accepts the invocation and uses JSON result output and debug diagnostics for the download operation

#### Scenario: User relies on environment token defaults
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md` with `GITHUB_TOKEN` or `GH_TOKEN` set and no CLI token or configuration-file token is present
- **THEN** the CLI uses the environment token automatically without requiring `--token`

#### Scenario: User runs the CLI without arguments
- **WHEN** a user runs `gh-download` without any additional arguments
- **THEN** the CLI prints the help output for the effective language and exits successfully without a missing-arguments error

### Requirement: CLI selects user-facing language from explicit configuration or locale
The CLI SHALL default its user-facing messages to English. It MUST switch to Chinese when the effective locale indicates Chinese, using `LC_ALL`, then `LC_MESSAGES`, then `LANG` as detection inputs. An explicit `--lang` option MUST take precedence over both configuration-file language and locale detection. When `--lang` is absent, a configuration-file `lang` value MUST take precedence over locale detection.

#### Scenario: No explicit language, no config language, and no Chinese locale
- **WHEN** a user runs the CLI without `--lang`, without a configuration-file `lang`, and the effective locale does not indicate Chinese
- **THEN** the CLI prints help text, status messages, and error guidance in English

#### Scenario: Chinese locale is configured without config language
- **WHEN** a user runs the CLI without `--lang`, without a configuration-file `lang`, and `LC_ALL`, `LC_MESSAGES`, or `LANG` indicates a Chinese locale
- **THEN** the CLI prints help text, status messages, and error guidance in Chinese

#### Scenario: Config language overrides locale
- **WHEN** a user runs the CLI without `--lang`, the configuration file contains `lang = "en"`, and the locale indicates Chinese
- **THEN** the CLI prints help text, status messages, and error guidance in English

#### Scenario: Explicit language overrides config language
- **WHEN** a user runs the CLI with `--lang zh` and the configuration file contains `lang = "en"`
- **THEN** the CLI prints help text, status messages, and error guidance in Chinese

### Requirement: CLI detects file versus directory targets from GitHub metadata
The CLI SHALL query GitHub metadata for the requested remote path before downloading content. It MUST distinguish between file and directory targets and it MUST fail with a user-visible error when GitHub returns an unsupported or unexpected target type. When the effective `api_base` is provided by either `--api-base` or the configuration file, the metadata request MUST use that value instead of the default public GitHub API base.

#### Scenario: Remote path resolves to a file
- **WHEN** the requested remote path points to a file in the repository
- **THEN** the CLI treats the request as a file download and resolves a single local output target

#### Scenario: Remote path resolves to a directory
- **WHEN** the requested remote path points to a directory in the repository
- **THEN** the CLI treats the request as a directory download and prepares to enumerate its contents recursively

#### Scenario: Custom API base is used for metadata detection from CLI
- **WHEN** a user runs `gh-download owner/repo docs ./docs --api-base https://ghe.example.com/api/v3`
- **THEN** the CLI sends its GitHub contents metadata requests to `https://ghe.example.com/api/v3` instead of `https://api.github.com`

#### Scenario: Custom API base is used for metadata detection from config
- **WHEN** a user runs `gh-download owner/repo docs ./docs` and the active configuration file contains `api_base = "https://ghe.example.com/api/v3"`
- **THEN** the CLI sends its GitHub contents metadata requests to `https://ghe.example.com/api/v3` instead of `https://api.github.com`

### Requirement: CLI enforces raw-download proxy boundaries
The CLI SHALL ignore ambient system proxy environment variables for its direct HTTP(S) requests. GitHub metadata API requests MUST NOT be retried through URL-prefix proxies such as `gh-proxy`, including when the effective `api_base` comes from a configuration file or `--api-base`. Anonymous raw file downloads SHALL follow the explicit prefix-proxy mode behavior defined by the `prefix-proxy-mode` capability. The CLI MUST NOT forward authentication credentials from CLI arguments, configuration files, or environment variables to the public proxy path.

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
- **WHEN** a request is made with a token from `--token`, the configuration file, `GITHUB_TOKEN`, or `GH_TOKEN`
- **THEN** the CLI does not send that credential through the proxy fallback path
