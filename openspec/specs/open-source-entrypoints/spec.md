# open-source-entrypoints Specification

## Purpose
TBD - created by archiving change improve-open-source-entrypoints. Update Purpose after archive.
## Requirements
### Requirement: Repository exposes concise multilingual README entrypoints
The repository SHALL maintain both `README.md` and `README.zh.md` as primary user entrypoints. Each README MUST present the project purpose, status badges, installation entrypoints, and a minimal file download example near the top of the document before detailed configuration and output reference sections.

#### Scenario: User opens the English README
- **WHEN** a user opens `README.md`
- **THEN** the top section presents the project purpose, status badges, installation entrypoints, and a minimal `gh-download` example before detailed option reference content

#### Scenario: User opens the Chinese README
- **WHEN** a user opens `README.zh.md`
- **THEN** the top section presents semantically equivalent Chinese project purpose, status badges, installation entrypoints, and a minimal `gh-download` example before detailed option reference content

### Requirement: Repository provides lightweight contribution and support files
The repository SHALL provide root-level `CONTRIBUTING.md`, `SUPPORT.md`, `SECURITY.md`, and `CODE_OF_CONDUCT.md` files that describe how users contribute changes, get support, report security-sensitive issues, and participate respectfully.

#### Scenario: Contributor looks for contribution guidance
- **WHEN** a contributor opens the repository contribution entrypoint
- **THEN** the repository provides contribution guidance that covers issue creation, pull request expectations, local verification, documentation alignment, and OpenSpec alignment

#### Scenario: User looks for security reporting guidance
- **WHEN** a user needs to report a security-sensitive issue
- **THEN** the repository provides a security policy that tells the user not to disclose token or credential details in a public issue and explains how to report privately

### Requirement: Repository provides actionable issue templates
The repository SHALL provide GitHub issue templates for bug reports and feature requests. Bug reports MUST collect enough information to reproduce CLI download problems, including version, operating system, installation method, command, repository path, local target, authentication mode, proxy mode, expected behavior, actual behavior, and redacted logs when available.

#### Scenario: User opens a bug report
- **WHEN** a user selects the bug report template
- **THEN** the template prompts for reproduction and environment details needed to investigate a `gh-download` CLI issue

#### Scenario: User opens a feature request
- **WHEN** a user selects the feature request template
- **THEN** the template prompts for the use case, expected behavior, and alternatives considered

### Requirement: Repository provides a focused pull request template
The repository SHALL provide a pull request template that prompts contributors to describe the change, list validation performed, and state whether CLI behavior, documentation, release behavior, or OpenSpec specs were changed.

#### Scenario: Contributor opens a pull request
- **WHEN** a contributor opens a pull request
- **THEN** the pull request body starts with a template that asks for change summary, verification, and product-surface impact checks

