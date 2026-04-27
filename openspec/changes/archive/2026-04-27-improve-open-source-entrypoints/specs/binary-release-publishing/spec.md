## MODIFIED Requirements

### Requirement: Release assets are packaged with repository metadata
The release workflow SHALL package Unix targets as `.tar.gz` archives and Windows targets as `.zip` archives. Each archive MUST contain the platform-specific `gh-download` executable, `README.md`, `README.zh.md`, and `LICENSE`.

#### Scenario: Unix asset is prepared
- **WHEN** a Linux or macOS target is packaged
- **THEN** the workflow creates a `.tar.gz` archive containing the executable, `README.md`, `README.zh.md`, and `LICENSE`

#### Scenario: Windows asset is prepared
- **WHEN** the Windows target is packaged
- **THEN** the workflow creates a `.zip` archive containing `gh-download.exe`, `README.md`, `README.zh.md`, and `LICENSE`
