## MODIFIED Requirements

### Requirement: Repository exposes concise multilingual README entrypoints
The repository SHALL maintain both `README.md` and `README.zh.md` as primary user entrypoints. Each README MUST present the project purpose, status badges, installation entrypoints including the Homebrew tap command, and a minimal file download example near the top of the document before detailed configuration and output reference sections.

#### Scenario: User opens the English README
- **WHEN** a user opens `README.md`
- **THEN** the top section presents the project purpose, status badges, Homebrew and alternative installation entrypoints, and a minimal `gh-download` example before detailed option reference content

#### Scenario: User opens the Chinese README
- **WHEN** a user opens `README.zh.md`
- **THEN** the top section presents semantically equivalent Chinese project purpose, status badges, Homebrew and alternative installation entrypoints, and a minimal `gh-download` example before detailed option reference content
