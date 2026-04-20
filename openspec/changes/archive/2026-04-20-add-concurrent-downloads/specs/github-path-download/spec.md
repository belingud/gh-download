## MODIFIED Requirements

### Requirement: CLI accepts GitHub path download inputs
The `gh-download` CLI SHALL accept a repository identifier, a repository-relative remote path, and a local target path as positional arguments when a download invocation is provided. When the CLI is invoked without any user-provided arguments, it SHALL print the localized help text and exit successfully instead of reporting missing required positional arguments. The CLI SHALL also support `--ref`, `--token`, `--proxy-base`, `--prefix-mode`, `--concurrency`, `-c`, `--debug`, and `--lang` options, and it MUST use `GITHUB_TOKEN` or `GH_TOKEN` as the default token source when `--token` is not provided.

#### Scenario: User provides explicit CLI arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --ref main --token abc --proxy-base https://gh-proxy.com/ --lang en`
- **THEN** the CLI accepts the invocation and uses the provided repository, remote path, local target, ref, token, proxy base, and language for the download operation

#### Scenario: User provides explicit prefix mode, concurrency, and debug arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --prefix-mode prefer --concurrency 8 --debug`
- **THEN** the CLI accepts the invocation and uses the provided prefix proxy mode, directory download concurrency, and debug flag for the download operation

#### Scenario: User relies on environment token defaults
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md` with `GITHUB_TOKEN` or `GH_TOKEN` set
- **THEN** the CLI uses the environment token automatically without requiring `--token`

#### Scenario: User runs the CLI without arguments
- **WHEN** a user runs `gh-download` without any additional arguments
- **THEN** the CLI prints the help output for the effective language and exits successfully without a missing-arguments error

### Requirement: CLI recursively downloads directory contents and preserves relative paths
For directory downloads, the CLI SHALL recursively enumerate all nested files beneath the requested remote path and write them using paths relative to the requested directory root. After enumeration, the CLI SHALL download directory files using at most the configured directory download concurrency. The CLI MUST create parent directories as needed before writing files. The CLI MUST warn and skip unsupported entries such as non-file, non-directory content returned by GitHub metadata.

#### Scenario: Nested repository files are downloaded with preserved relative paths
- **WHEN** the requested remote directory contains nested subdirectories and files
- **THEN** the CLI downloads every file below that directory, preserves the relative path structure inside the local output directory, and applies the configured directory download concurrency during file transfers

#### Scenario: Unsupported entry is encountered
- **WHEN** GitHub metadata includes an entry type that is not a regular file or directory
- **THEN** the CLI skips that entry and prints a warning identifying the skipped repository path and entry type
