## MODIFIED Requirements

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
