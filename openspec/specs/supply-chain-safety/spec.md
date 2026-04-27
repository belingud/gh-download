# supply-chain-safety Specification

## Purpose
TBD - created by archiving change add-supply-chain-safety. Update Purpose after archive.
## Requirements
### Requirement: Repository configures dependency version updates
The repository SHALL provide a `.github/dependabot.yml` file that configures Dependabot version updates for Cargo dependencies and GitHub Actions workflow dependencies.

#### Scenario: Cargo dependency updates are configured
- **WHEN** Dependabot reads the repository configuration
- **THEN** it has a `cargo` update entry for the repository root with a scheduled interval

#### Scenario: GitHub Actions dependency updates are configured
- **WHEN** Dependabot reads the repository configuration
- **THEN** it has a `github-actions` update entry for the repository root with a scheduled interval

### Requirement: Repository audits Rust dependency advisories
The repository SHALL provide an independent GitHub Actions workflow for Rust dependency security audits. The workflow MUST run on dependency-related push and pull request changes, on a recurring schedule, and by manual dispatch.

#### Scenario: Dependency files change
- **WHEN** a push or pull request changes `Cargo.toml`, `Cargo.lock`, or the audit workflow file
- **THEN** GitHub Actions starts the Rust dependency audit workflow

#### Scenario: New advisory is published after dependencies remain unchanged
- **WHEN** the scheduled audit time is reached
- **THEN** GitHub Actions starts the Rust dependency audit workflow even if dependency files have not changed

#### Scenario: Maintainer triggers an audit manually
- **WHEN** a maintainer starts the audit workflow through `workflow_dispatch`
- **THEN** GitHub Actions starts the Rust dependency audit workflow for the selected revision

### Requirement: Supply-chain checks remain separate from regular CI
The repository SHALL keep supply-chain auditing separate from the regular validation workflow that runs `just check`.

#### Scenario: Regular CI runs
- **WHEN** the regular validation workflow executes for an ordinary code or documentation change
- **THEN** it remains responsible for `just check` and does not run the Rust dependency audit as part of that workflow

