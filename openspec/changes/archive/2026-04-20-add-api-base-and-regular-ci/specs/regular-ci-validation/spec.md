## ADDED Requirements

### Requirement: Repository validates changes on push and pull request
The repository SHALL define a GitHub Actions workflow that runs on ordinary `push` and `pull_request` events so that non-release changes are validated before a version tag is published.

#### Scenario: Branch push triggers regular validation
- **WHEN** a contributor pushes a non-tag commit to the repository
- **THEN** GitHub Actions starts the regular validation workflow for that revision

#### Scenario: Pull request triggers regular validation
- **WHEN** a contributor opens or updates a pull request against the repository
- **THEN** GitHub Actions starts the regular validation workflow for the pull request revision

### Requirement: Regular validation runs the repository standard verification command
The regular validation workflow SHALL install the project toolchain needed for the repository and execute `just check` as the standard verification command.

#### Scenario: Regular CI run executes standard checks
- **WHEN** the regular validation workflow runs
- **THEN** it installs stable Rust plus the `just` command and executes `just check`

### Requirement: Regular validation remains separate from tag-driven release publishing
The regular validation workflow SHALL remain distinct from the tag-triggered release workflow so that ordinary validation does not package or publish release artifacts.

#### Scenario: Regular CI run does not publish release assets
- **WHEN** the regular validation workflow executes for a branch push or pull request
- **THEN** it performs verification only and does not build release archives or publish a GitHub Release
