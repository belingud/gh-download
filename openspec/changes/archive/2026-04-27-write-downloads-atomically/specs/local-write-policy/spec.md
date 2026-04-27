## MODIFIED Requirements

### Requirement: CLI supports explicit overwrite mode
The CLI SHALL support an explicit `--overwrite` flag that replaces existing local files instead of skipping them. When the flag is enabled, the CLI SHALL use the same resolved local file targets it would otherwise skip. Replacement downloads MUST write to a temporary file first and MUST NOT truncate or replace the existing final file before the replacement transfer body has been copied successfully.

#### Scenario: Overwrite mode replaces an existing direct file target
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --overwrite` and the local file already exists
- **THEN** the CLI replaces the existing local file with the downloaded content

#### Scenario: Overwrite mode replaces existing files during a directory download
- **WHEN** a user runs `gh-download owner/repo src ./downloads --overwrite` and one or more resolved local file targets already exist
- **THEN** the CLI replaces those existing local files while continuing to preserve the directory-relative output structure

#### Scenario: Failed overwrite transfer does not truncate an existing file
- **WHEN** a replacement download runs with `--overwrite`, the resolved local file target exists, and the replacement transfer fails while copying the HTTP response body
- **THEN** the CLI leaves the existing local file unchanged and reports the failure
