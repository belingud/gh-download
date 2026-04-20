# Capability: local-write-policy

## Purpose

Define how the CLI handles existing local files when a download resolves to a path that is already present on disk.

## Requirements

### Requirement: Existing local files are skipped by default
When the final resolved local file target already exists, the CLI SHALL skip writing that file by default instead of overwriting it. This rule SHALL apply to both direct file downloads and per-file writes inside directory downloads.

#### Scenario: Direct file target already exists
- **WHEN** a user downloads a remote file and the resolved local file target already exists
- **THEN** the CLI leaves the existing local file unchanged and reports that the file was skipped

#### Scenario: Directory download encounters an existing local file
- **WHEN** a directory download resolves a nested local file target that already exists
- **THEN** the CLI leaves that existing local file unchanged and continues the overall directory download

### Requirement: CLI supports explicit overwrite mode
The CLI SHALL support an explicit `--overwrite` flag that replaces existing local files instead of skipping them. When the flag is enabled, the CLI SHALL use the same resolved local file targets it would otherwise skip.

#### Scenario: Overwrite mode replaces an existing direct file target
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --overwrite` and the local file already exists
- **THEN** the CLI replaces the existing local file with the downloaded content

#### Scenario: Overwrite mode replaces existing files during a directory download
- **WHEN** a user runs `gh-download owner/repo src ./downloads --overwrite` and one or more resolved local file targets already exist
- **THEN** the CLI replaces those existing local files while continuing to preserve the directory-relative output structure

### Requirement: CLI reports skipped existing files separately from unsupported entries
The CLI SHALL report skipped-existing local files distinctly from unsupported repository entries returned by GitHub metadata. Completion output MUST allow users to distinguish downloaded files, skipped existing files, and skipped unsupported entries.

#### Scenario: Completion summary includes skipped existing files
- **WHEN** a download completes with one or more existing local files skipped
- **THEN** the CLI completion output identifies the count of skipped existing files separately from the count of skipped unsupported entries
