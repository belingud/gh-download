## ADDED Requirements

### Requirement: CLI accepts GitHub path download inputs
The `gh-download` CLI SHALL accept a repository identifier, a repository-relative remote path, and a local target path as positional arguments. The CLI SHALL also support `--ref`, `--token`, `--proxy-base`, and `--lang` options, and it MUST use `GITHUB_TOKEN` or `GH_TOKEN` as the default token source when `--token` is not provided.

#### Scenario: User provides explicit CLI arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --ref main --token abc --proxy-base https://gh-proxy.com/ --lang en`
- **THEN** the CLI accepts the invocation and uses the provided repository, remote path, local target, ref, token, proxy base, and language for the download operation

#### Scenario: User relies on environment token defaults
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md` with `GITHUB_TOKEN` or `GH_TOKEN` set
- **THEN** the CLI uses the environment token automatically without requiring `--token`

### Requirement: CLI selects user-facing language from explicit configuration or locale
The CLI SHALL default its user-facing messages to English. It MUST switch to Chinese when the effective locale indicates Chinese, using `LC_ALL`, then `LC_MESSAGES`, then `LANG` as detection inputs. An explicit `--lang` option MUST take precedence over locale detection.

#### Scenario: No explicit language and no Chinese locale
- **WHEN** a user runs the CLI without `--lang` and the effective locale does not indicate Chinese
- **THEN** the CLI prints help text, status messages, and error guidance in English

#### Scenario: Chinese locale is configured
- **WHEN** a user runs the CLI without `--lang` and `LC_ALL`, `LC_MESSAGES`, or `LANG` indicates a Chinese locale
- **THEN** the CLI prints help text, status messages, and error guidance in Chinese

#### Scenario: Explicit language overrides locale
- **WHEN** a user runs the CLI with `--lang en` while the locale indicates Chinese
- **THEN** the CLI prints help text, status messages, and error guidance in English

### Requirement: CLI detects file versus directory targets from GitHub metadata
The CLI SHALL query GitHub metadata for the requested remote path before downloading content. It MUST distinguish between file and directory targets and it MUST fail with a user-visible error when GitHub returns an unsupported or unexpected target type.

#### Scenario: Remote path resolves to a file
- **WHEN** the requested remote path points to a file in the repository
- **THEN** the CLI treats the request as a file download and resolves a single local output target

#### Scenario: Remote path resolves to a directory
- **WHEN** the requested remote path points to a directory in the repository
- **THEN** the CLI treats the request as a directory download and prepares to enumerate its contents recursively

### Requirement: CLI downloads files and directories with deterministic local path handling
The CLI SHALL stream downloaded file bytes directly to disk instead of buffering the full payload in memory. For a remote file, the local target MUST support either a direct file path or an existing destination directory. For a remote directory, the CLI MUST recreate the remote directory name under the local target unless the local target already ends with the same directory name.

#### Scenario: File target points to an existing directory
- **WHEN** a user downloads a remote file and the local target already exists as a directory
- **THEN** the CLI writes the file into that directory using the remote file name

#### Scenario: Directory target would otherwise double-nest
- **WHEN** a user downloads a remote directory and the local target already ends with the same directory name
- **THEN** the CLI reuses the provided directory path instead of nesting the same name twice

### Requirement: CLI recursively downloads directory contents and preserves relative paths
For directory downloads, the CLI SHALL recursively enumerate all nested files beneath the requested remote path and write them using paths relative to the requested directory root. The CLI MUST create parent directories as needed before writing files. The CLI MUST warn and skip unsupported entries such as non-file, non-directory content returned by GitHub metadata.

#### Scenario: Nested repository files are downloaded
- **WHEN** the requested remote directory contains nested subdirectories and files
- **THEN** the CLI downloads every file below that directory and preserves the relative path structure inside the local output directory

#### Scenario: Unsupported entry is encountered
- **WHEN** GitHub metadata includes an entry type that is not a regular file or directory
- **THEN** the CLI skips that entry and prints a warning identifying the skipped repository path and entry type

### Requirement: CLI retries anonymous requests through the configured proxy when direct access fails
When the CLI is running without an authentication token and a proxy base is configured, it SHALL retry eligible GitHub metadata or raw file requests through the configured proxy after direct GitHub rate-limit, server-side, or network failures. The CLI MUST NOT forward authentication credentials to the public proxy path.

#### Scenario: Anonymous GitHub API request is rate limited
- **WHEN** a direct anonymous GitHub request fails with a retryable HTTP status such as `403`, `429`, or a transient server error
- **THEN** the CLI retries the request through the configured proxy and informs the user that proxy fallback is being used

#### Scenario: Authenticated request fails
- **WHEN** a request is made with an explicit token or an environment token
- **THEN** the CLI does not send that credential through the proxy fallback path

### Requirement: CLI provides concise colored status output and actionable failure guidance
The CLI SHALL print a startup summary that includes the repository, ref selection, remote path, and local target. It SHALL print concise progress messages during downloads and a completion summary at the end of a successful operation. On failure, the CLI MUST present a short explanation plus at least one remediation suggestion for common categories including authentication, missing path or ref, network failure, and local filesystem write failure.

#### Scenario: Successful download reports progress and completion
- **WHEN** a download completes successfully
- **THEN** the CLI shows a readable progress trail and a final success summary that identifies the saved local path

#### Scenario: Download fails due to missing authentication
- **WHEN** GitHub rejects a request because authentication is missing or anonymous access is rate limited
- **THEN** the CLI reports the failure in user-facing terms and suggests providing `--token` or setting `GITHUB_TOKEN` or `GH_TOKEN`
