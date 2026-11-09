## MODIFIED Requirements

### Requirement: CLI accepts GitHub path download inputs
The `gh-download` CLI SHALL accept a repository identifier, a repository-relative remote path, and a local target path as positional arguments when a download invocation is provided. When the CLI is invoked without any user-provided arguments, it SHALL print the localized help text and exit successfully instead of reporting missing required positional arguments. The CLI SHALL also support `--ref`, `--token`, `--api-base`, `--proxy-base`, `--prefix-mode`, `--concurrency`, `-c`, `--overwrite`, `--keep-going`, `--symlinks`, `--lfs`, `--json`, `--debug`, `--lang`, and `--config` options. The `--symlinks` option MUST accept `auto`, `link`, and `skip`; the `--lfs` option MUST accept `error` and `media`. When `--token` is not provided, the CLI MUST resolve the effective token from the configuration file before falling back to `GITHUB_TOKEN` or `GH_TOKEN`.

#### Scenario: User provides explicit CLI arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --ref main --token abc --api-base https://ghe.example.com/api/v3 --proxy-base https://gh-proxy.com/ --lang en`
- **THEN** the CLI accepts the invocation and uses the provided repository, remote path, local target, ref, token, API base, proxy base, and language for the download operation

#### Scenario: User relies on configuration file defaults
- **WHEN** a user runs `gh-download owner/repo src ./downloads` and the active configuration file contains `api_base`, `token`, and `lang`
- **THEN** the CLI uses those configuration-file values for the invocation without requiring the matching CLI options

#### Scenario: User selects directory transfer recovery options
- **WHEN** a user runs `gh-download owner/repo src ./downloads --keep-going --symlinks skip --lfs media`
- **THEN** the CLI accepts the invocation and applies fail-soft directory transfer, symlink skipping, and LFS media retrieval for that transfer

#### Scenario: User provides explicit machine-readable output and debug arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --json --debug`
- **THEN** the CLI accepts the invocation and uses JSON result output and debug diagnostics for the download operation

#### Scenario: User relies on environment token defaults
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md` with `GITHUB_TOKEN` or `GH_TOKEN` set and no CLI token or configuration-file token is present
- **THEN** the CLI uses the environment token automatically without requiring `--token`

#### Scenario: User runs the CLI without arguments
- **WHEN** a user runs `gh-download` without any additional arguments
- **THEN** the CLI prints the help output for the effective language and exits successfully without a missing-arguments error

### Requirement: CLI detects file versus directory targets from GitHub metadata
The CLI SHALL query GitHub metadata for the requested remote path before downloading content. It MUST distinguish between file and directory targets and it MUST fail with a user-visible error when GitHub returns an unsupported or unexpected target type. The metadata request MUST use the resolved commit revision defined by the immutable commit requirement, and when the effective `api_base` is provided by either `--api-base` or the configuration file, the metadata request MUST use that value instead of the default public GitHub API base.

#### Scenario: Remote path resolves to a file
- **WHEN** the requested remote path points to a file in the repository
- **THEN** the CLI treats the request as a file download and resolves a single local output target

#### Scenario: Remote path resolves to a directory
- **WHEN** the requested remote path points to a directory in the repository
- **THEN** the CLI treats the request as a directory download and prepares to enumerate its contents recursively from tree metadata

#### Scenario: Custom API base is used for metadata detection from CLI
- **WHEN** a user runs `gh-download owner/repo docs ./docs --api-base https://ghe.example.com/api/v3`
- **THEN** the CLI sends its GitHub metadata requests to `https://ghe.example.com/api/v3` instead of `https://api.github.com`

#### Scenario: Custom API base is used for metadata detection from config
- **WHEN** a user runs `gh-download owner/repo docs ./docs` and the active configuration file contains `api_base = "https://ghe.example.com/api/v3"`
- **THEN** the CLI sends its GitHub metadata requests to `https://ghe.example.com/api/v3` instead of `https://api.github.com`

### Requirement: CLI downloads files and directories with deterministic local path handling
The CLI SHALL stream downloaded file bytes directly to a local temporary file instead of buffering the full payload in memory or writing an incomplete transfer directly to the final target. The CLI MUST commit the temporary file to the final target only after the HTTP response body has been copied successfully. If the response body copy fails, the CLI MUST NOT leave partially copied bytes at a final target that did not exist before the failed transfer, and it MUST remove the temporary file where possible. For a remote file, the local target MUST support either a direct file path or an existing destination directory. A trailing path separator on the local target SHALL be treated as directory intent: for file downloads the CLI MUST create that directory when needed and write the file inside it, and for directory downloads the target remains the parent directory. For a remote directory, the CLI MUST recreate the remote directory name under the local target unless the local target already ends with the same directory name. Before path resolution is displayed or joined with remote output names, the CLI SHALL lexically normalize local target paths so redundant `.` segments are removed without requiring the path to already exist on disk. When a file name is derived from GitHub metadata, the CLI MUST reject names that are absolute, contain path separators, or contain current-directory or parent-directory segments; it SHALL fall back to the validated remote path file name and MUST fail with a user-visible error when no safe file name can be derived.

#### Scenario: File target points to an existing directory
- **WHEN** a user downloads a remote file and the local target already exists as a directory
- **THEN** the CLI writes the file into that directory using the validated remote file name

#### Scenario: File target uses a trailing separator for a new directory
- **WHEN** a user runs `gh-download owner/repo README.md ./downloads/` and `downloads` does not exist yet
- **THEN** the CLI creates the `downloads` directory and writes `README.md` inside it instead of writing a file named `downloads`

#### Scenario: Unsafe metadata file name is rejected
- **WHEN** GitHub metadata for a file download reports a name containing `..` or a path separator
- **THEN** the CLI does not write outside the local target and fails or falls back to a safe remote path file name as defined above

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
For directory downloads, the CLI SHALL recursively enumerate all nested regular files beneath the requested remote path from GitHub tree metadata and write them using paths relative to the requested directory root. Enumeration MUST NOT depend on the number of directories below the requested path: the CLI SHALL use at most one recursive tree metadata request when GitHub returns the full tree, and when GitHub reports that tree response as truncated, the CLI SHALL fall back to non-recursive per-subtree tree requests until the complete tree is enumerated. The CLI MUST validate each enumerated file path before joining it with the local output directory. If an enumerated file path is outside the requested remote directory root, or its directory-relative output path is empty, absolute, contains current-directory or parent-directory segments, or contains platform separator aliases, the CLI MUST fail before writing that file. After enumeration, the CLI SHALL download directory files using at most the configured directory download concurrency. The CLI MUST create parent directories as needed before writing files. Non-regular entries SHALL be handled according to the non-regular entry requirement instead of being treated as regular files.

#### Scenario: Nested repository files are downloaded with preserved relative paths
- **WHEN** the requested remote directory contains nested subdirectories and files
- **THEN** the CLI downloads every file below that directory, preserves the relative path structure inside the local output directory, and applies the configured directory download concurrency during file transfers

#### Scenario: Large directory is enumerated completely
- **WHEN** the requested remote directory contains more than 1,000 entries
- **THEN** the CLI enumerates the complete tree and downloads every regular file instead of silently stopping at the GitHub Contents API directory limit

#### Scenario: Truncated tree response falls back to per-subtree requests
- **WHEN** GitHub reports that the recursive tree response is truncated
- **THEN** the CLI continues enumeration with non-recursive per-subtree tree requests until the full tree below the requested path is resolved

#### Scenario: Unsupported entry is encountered
- **WHEN** GitHub tree metadata includes an entry kind that is not a regular file or directory
- **THEN** the CLI handles that entry according to the non-regular entry requirement, warns with the skipped repository path and entry kind, and continues the directory transfer

#### Scenario: Directory metadata path attempts to leave the output directory
- **WHEN** a directory download enumerates a file path whose directory-relative output path contains `..` or is absolute
- **THEN** the CLI fails before writing that file outside the selected local output directory

#### Scenario: Directory metadata path is outside the requested remote root
- **WHEN** a directory download for `src` receives an enumerated file path that is not beneath `src/`
- **THEN** the CLI fails before writing that file

### Requirement: CLI provides concise colored status output and actionable failure guidance
The CLI SHALL print a structured startup summary with separators that includes the repository, ref selection, remote path, and local target. For directory downloads, it SHALL print the discovered file count, the effective worker thread count used for that transfer, the remote directory, and the created local directory before file progress. It SHALL print concise per-file download progress messages, including when an existing local entry is skipped, and a structured completion summary at the end of a successful operation. ANSI color MUST be used only when stdout is an interactive terminal and the `NO_COLOR` environment variable is not set; `--no-color` MUST always disable color. On failure, the CLI MUST present a short explanation plus at least one remediation suggestion for common categories including authentication, missing path or ref, network failure, and local filesystem write failure. A failed file download MUST identify the remote path, the resolved local path, and the attempted download strategies, and when some files were already saved the CLI MUST report the partial progress counts. When JSON output mode is enabled, these human-readable progress, completion, and error messages SHALL be suppressed from stdout in favor of the machine-readable JSON result.

#### Scenario: Successful download reports progress and completion
- **WHEN** a download completes successfully without JSON mode
- **THEN** the CLI shows a readable progress trail and a final success summary that identifies the saved local path

#### Scenario: Directory startup reports effective worker thread count
- **WHEN** a directory download starts without JSON mode
- **THEN** the CLI shows the discovered file count, the remote directory, and the effective worker thread count that will be used for the transfer

#### Scenario: Existing local file is skipped
- **WHEN** the CLI skips writing a resolved local file because an entry already exists at that path and overwrite mode is not enabled
- **THEN** the progress output identifies that file as skipped rather than downloaded

#### Scenario: Non-interactive output disables color
- **WHEN** stdout is redirected to a file or pipe, or `NO_COLOR` is set in the environment
- **THEN** the CLI emits status text without ANSI escape sequences

#### Scenario: Failed directory file identifies the failing target
- **WHEN** a file inside a directory transfer fails
- **THEN** the error output identifies the remote path, the resolved local path, the attempted strategies, and the number of files already saved

#### Scenario: JSON mode suppresses human-readable stdout output
- **WHEN** a user enables JSON output mode for a download
- **THEN** the CLI does not print the default human-readable startup, progress, completion, or error text to stdout

#### Scenario: Anonymous fallback proxy retry is used
- **WHEN** the CLI retries an eligible anonymous raw file download through `--proxy-base`
- **THEN** the warning output identifies the full generated fallback URL and redacts any embedded credentials before printing it

#### Scenario: Direct raw file download fails before Raw API fallback
- **WHEN** a direct file raw download attempt fails and the CLI continues to the next download strategy
- **THEN** the CLI reports the failure of each attempted strategy in the final error instead of only the last attempt

#### Scenario: Download fails due to missing authentication
- **WHEN** GitHub rejects a request because authentication is missing or anonymous access is rate limited
- **THEN** the CLI reports the failure in user-facing terms and suggests providing `--token` or setting `GITHUB_TOKEN` or `GH_TOKEN`

## ADDED Requirements

### Requirement: CLI pins remote content to an immutable commit revision
Before enumerating or downloading content, the CLI SHALL resolve the effective ref to a full commit SHA. The effective ref is `--ref` when provided, and the repository default branch otherwise. Directory enumeration, file downloads, and debug and JSON diagnostics MUST use that resolved commit SHA so that a branch moving during one invocation cannot produce a local copy that mixes revisions. When `--ref` is already a full commit SHA, the CLI MAY use it directly without a separate ref-resolution request. When the effective ref cannot be resolved, the CLI MUST fail with a user-visible error that identifies the requested ref.

#### Scenario: Branch moves during a directory transfer
- **WHEN** a directory download starts from a branch ref and that branch receives new commits while the transfer is running
- **THEN** every file saved by that invocation comes from the commit resolved at the start of the invocation

#### Scenario: Explicit ref is already a commit SHA
- **WHEN** a user runs a download with `--ref` set to a full commit SHA
- **THEN** the CLI uses that SHA for enumeration and downloads

#### Scenario: Default branch is resolved when ref is omitted
- **WHEN** a user runs a download without `--ref`
- **THEN** the CLI resolves the repository default branch to a commit SHA and uses that SHA for enumeration and downloads

#### Scenario: Unresolvable ref fails before downloading
- **WHEN** the requested `--ref` does not resolve to a commit in the repository
- **THEN** the CLI fails with a user-visible error that identifies the requested ref instead of downloading mixed or partial content

### Requirement: CLI classifies non-regular entries and Git LFS objects
For directory downloads, the CLI SHALL determine each enumerated entry kind from immutable tree metadata rather than trusting a directory listing type field. Symlinks SHALL be handled according to `--symlinks`: `auto` (the default) SHALL save the target content when the symlink resolves to a regular file and SHALL skip the entry with a warning when it resolves to a directory or submodule; `skip` SHALL skip every symlink with a warning; `link` SHALL create a local symbolic link whose target is the repository symlink target on platforms that support symbolic links, and MUST fail that entry with an actionable error on platforms that do not. Submodule entries SHALL be skipped with a warning that includes the submodule repository URL and pinned commit. After streaming any file content, the CLI SHALL detect Git LFS pointer content: with `--lfs error` (the default) it MUST fail that file with an actionable error and MUST NOT commit pointer text as the downloaded file; with `--lfs media` it SHALL attempt to fetch the LFS object from the GitHub media host and MUST fall back to the actionable error when that host is unavailable. LFS media retrieval is only defined for the default public GitHub API base; on a custom `--api-base` the CLI MUST report that LFS retrieval is unsupported instead of downloading pointer text. When the requested remote path itself resolves to a submodule or a symlink that does not resolve to a regular file, the CLI MUST fail with an actionable error identifying the entry kind.

#### Scenario: Symlink to a regular file is followed
- **WHEN** a directory transfer encounters a symlink whose target is a regular file and `--symlinks auto` is effective
- **THEN** the CLI saves the target file content at the symlink's relative path

#### Scenario: Symlink to a directory is skipped
- **WHEN** a directory transfer encounters a symlink whose target is a directory and `--symlinks auto` is effective
- **THEN** the CLI skips the entry, prints a warning that identifies the symlink path and target, and does not save target path text as file content

#### Scenario: User requests symlink materialization
- **WHEN** a user runs a directory download with `--symlinks link` on a platform that supports symbolic links
- **THEN** the CLI creates local symbolic links with the repository symlink targets instead of saving regular files

#### Scenario: Submodule is skipped with actionable details
- **WHEN** a directory transfer encounters a submodule entry
- **THEN** the CLI skips the entry and warns with the submodule repository URL and pinned commit so the user can obtain it separately

#### Scenario: Git LFS pointer content is rejected by default
- **WHEN** a downloaded file's content is a Git LFS pointer and `--lfs error` is effective
- **THEN** the CLI does not commit the pointer text as the downloaded file and fails that file with an actionable error describing the LFS object

#### Scenario: User opts into LFS media retrieval
- **WHEN** a downloaded file's content is a Git LFS pointer, `--lfs media` is effective, and the default public GitHub API base is in use
- **THEN** the CLI attempts to fetch the LFS object from the GitHub media host and saves the object content or reports an actionable failure

#### Scenario: Requested path is not a regular file
- **WHEN** the requested remote path resolves to a submodule or to a symlink that does not resolve to a regular file
- **THEN** the CLI fails with a user-visible error that identifies the entry kind and suggests a directory or clone-based alternative
