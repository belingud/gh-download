## MODIFIED Requirements

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
