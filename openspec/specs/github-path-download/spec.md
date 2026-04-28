# Capability: github-path-download

## Purpose

Define the core command-line contract and download behavior for fetching files or directories from a GitHub repository path.

## Requirements

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

### Requirement: CLI downloads files and directories with deterministic local path handling
The CLI SHALL stream downloaded file bytes directly to a local temporary file instead of buffering the full payload in memory or writing an incomplete transfer directly to the final target. The CLI MUST commit the temporary file to the final target only after the HTTP response body has been copied successfully. If the response body copy fails, the CLI MUST NOT leave partially copied bytes at a final target that did not exist before the failed transfer, and it MUST remove the temporary file where possible. For a remote file, the local target MUST support either a direct file path or an existing destination directory. For a remote directory, the CLI MUST recreate the remote directory name under the local target unless the local target already ends with the same directory name. Before path resolution is displayed or joined with remote output names, the CLI SHALL lexically normalize local target paths so redundant `.` segments are removed without requiring the path to already exist on disk.

#### Scenario: File target points to an existing directory
- **WHEN** a user downloads a remote file and the local target already exists as a directory
- **THEN** the CLI writes the file into that directory using the remote file name

#### Scenario: Directory target would otherwise double-nest
- **WHEN** a user downloads a remote directory and the local target already ends with the same directory name
- **THEN** the CLI reuses the provided directory path instead of nesting the same name twice

#### Scenario: Local target removes redundant current-directory segments
- **WHEN** a user runs `gh-download owner/repo .github ./var`
- **THEN** the CLI resolves and displays the local path as `<cwd>/var/.github` instead of preserving a redundant `./` segment inside the absolute path

#### Scenario: Failed transfer does not create an incomplete final file
- **WHEN** a file transfer fails while copying the HTTP response body and the resolved local file target did not exist before the transfer
- **THEN** the CLI does not leave the partially copied bytes at the resolved local file target

### Requirement: CLI recursively downloads directory contents and preserves relative paths
For directory downloads, the CLI SHALL recursively enumerate all nested files beneath the requested remote path and write them using paths relative to the requested directory root. The CLI MUST validate each enumerated file path before joining it with the local output directory. If an enumerated file path is outside the requested remote directory root, or its directory-relative output path is empty, absolute, contains current-directory or parent-directory segments, or contains platform separator aliases, the CLI MUST fail before writing that file. After enumeration, the CLI SHALL download directory files using at most the configured directory download concurrency. The CLI MUST create parent directories as needed before writing files. The CLI MUST warn and skip unsupported entries such as non-file, non-directory content returned by GitHub metadata.

#### Scenario: Nested repository files are downloaded with preserved relative paths
- **WHEN** the requested remote directory contains nested subdirectories and files
- **THEN** the CLI downloads every file below that directory, preserves the relative path structure inside the local output directory, and applies the configured directory download concurrency during file transfers

#### Scenario: Unsupported entry is encountered
- **WHEN** GitHub metadata includes an entry type that is not a regular file or directory
- **THEN** the CLI skips that entry and prints a warning identifying the skipped repository path and entry type

#### Scenario: Directory metadata path attempts to leave the output directory
- **WHEN** a directory download enumerates a file path whose directory-relative output path contains `..` or is absolute
- **THEN** the CLI fails before writing that file outside the selected local output directory

#### Scenario: Directory metadata path is outside the requested remote root
- **WHEN** a directory download for `src` receives an enumerated file path that is not beneath `src/`
- **THEN** the CLI fails before writing that file

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

### Requirement: CLI provides concise colored status output and actionable failure guidance
The CLI SHALL print a structured startup summary with separators that includes the repository, ref selection, remote path, and local target. For directory downloads, it SHALL print the discovered file count, the effective worker thread count used for that transfer, the remote directory, and the created local directory before file progress. It SHALL print concise per-file download progress messages, including when an existing local file is skipped, and a structured completion summary at the end of a successful operation. On failure, the CLI MUST present a short explanation plus at least one remediation suggestion for common categories including authentication, missing path or ref, network failure, and local filesystem write failure. When JSON output mode is enabled, these human-readable progress, completion, and error messages SHALL be suppressed from stdout in favor of the machine-readable JSON result.

#### Scenario: Successful download reports progress and completion
- **WHEN** a download completes successfully without JSON mode
- **THEN** the CLI shows a readable progress trail and a final success summary that identifies the saved local path

#### Scenario: Directory startup reports effective worker thread count
- **WHEN** a directory download starts without JSON mode
- **THEN** the CLI shows the discovered file count, the remote directory, and the effective worker thread count that will be used for the transfer

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
