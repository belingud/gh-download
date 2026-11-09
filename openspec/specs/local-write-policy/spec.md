# Capability: local-write-policy

## Purpose

Define how the CLI handles existing local files when a download resolves to a path that is already present on disk.

## Requirements

### Requirement: Existing local files are skipped by default
When an entry already exists at the final resolved local file target, the CLI SHALL skip writing that file by default instead of overwriting it. This rule SHALL apply to both direct file downloads and per-file writes inside directory downloads. Existence SHALL be evaluated for any filesystem entry at that path, including a regular file or a symbolic link, and not only for regular files. When the path exists as a directory, the CLI MUST fail with a user-visible error that the resolved file target is a directory instead of attempting to replace or rename over it.

#### Scenario: Direct file target already exists
- **WHEN** a user downloads a remote file and a regular file already exists at the resolved local file target
- **THEN** the CLI leaves the existing local file unchanged and reports that the file was skipped

#### Scenario: Existing symlink is skipped
- **WHEN** a user downloads a remote file and a symbolic link already exists at the resolved local file target
- **THEN** the CLI leaves the existing symbolic link unchanged and reports that the file was skipped

#### Scenario: Directory download encounters an existing local file
- **WHEN** a directory download resolves a nested local file target that already exists
- **THEN** the CLI leaves that existing local file unchanged and continues the overall directory download

#### Scenario: Directory exists at the resolved file target
- **WHEN** the resolved local file target for a file download already exists as a directory
- **THEN** the CLI fails with a clear error that the resolved target is a directory instead of attempting a replacement

### Requirement: CLI supports explicit overwrite mode
The CLI SHALL support an explicit `--overwrite` flag that replaces existing local entries instead of skipping them. When the flag is enabled, the CLI SHALL use the same resolved local file targets it would otherwise skip. Replacement downloads MUST write to a temporary file first and MUST NOT truncate or replace the existing final entry before the replacement transfer body has been copied successfully. A directory at the resolved file target MUST NOT be replaced by overwrite mode and MUST produce the same clear error as the default mode.

#### Scenario: Overwrite mode replaces an existing direct file target
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --overwrite` and the local file already exists
- **THEN** the CLI replaces the existing local file with the downloaded content

#### Scenario: Overwrite mode replaces an existing symlink
- **WHEN** a user runs a file download with `--overwrite` and a symbolic link already exists at the resolved local file target
- **THEN** the CLI replaces the symbolic link entry with the downloaded file

#### Scenario: Overwrite mode replaces existing files during a directory download
- **WHEN** a user runs `gh-download owner/repo src ./downloads --overwrite` and one or more resolved local file targets already exist
- **THEN** the CLI replaces those existing local files while continuing to preserve the directory-relative output structure

#### Scenario: Failed overwrite transfer does not truncate an existing file
- **WHEN** a replacement download runs with `--overwrite`, the resolved local file target exists, and the replacement transfer fails while copying the HTTP response body
- **THEN** the CLI leaves the existing local file unchanged and reports the failure

### Requirement: CLI reports skipped existing files separately from unsupported entries
The CLI SHALL report skipped-existing local files distinctly from unsupported repository entries returned by GitHub metadata. Completion output MUST allow users to distinguish downloaded files, skipped existing files, and skipped unsupported entries.

#### Scenario: Completion summary includes skipped existing files
- **WHEN** a download completes with one or more existing local files skipped
- **THEN** the CLI completion output identifies the count of skipped existing files separately from the count of skipped unsupported entries
