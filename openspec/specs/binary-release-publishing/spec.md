# Capability: binary-release-publishing

## Purpose

Define how tagged releases build, package, and publish distributable `gh-download` binaries.

## Requirements

### Requirement: Version tags trigger automated release builds
The repository SHALL define a GitHub Actions workflow that starts automatically when a Git tag matching `v*` is pushed. The workflow MUST build release artifacts from the tagged revision without requiring a manual release step.

#### Scenario: Version tag is pushed
- **WHEN** a maintainer pushes a tag such as `v0.1.0`
- **THEN** GitHub Actions starts the release workflow for that tagged commit

### Requirement: Release workflow builds binaries for the supported platform matrix
The release workflow SHALL build the `gh-download` binary for the following targets: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, and `x86_64-pc-windows-msvc`.

#### Scenario: Release build runs
- **WHEN** the tag-triggered release workflow executes successfully
- **THEN** it produces one compiled binary artifact for each supported target in the release matrix

### Requirement: Release assets are packaged with repository metadata
The release workflow SHALL package Unix targets as `.tar.gz` archives and Windows targets as `.zip` archives. Each archive MUST contain the platform-specific `gh-download` executable, `README.md`, and `LICENSE`.

#### Scenario: Unix asset is prepared
- **WHEN** a Linux or macOS target is packaged
- **THEN** the workflow creates a `.tar.gz` archive containing the executable, `README.md`, and `LICENSE`

#### Scenario: Windows asset is prepared
- **WHEN** the Windows target is packaged
- **THEN** the workflow creates a `.zip` archive containing `gh-download.exe`, `README.md`, and `LICENSE`

### Requirement: Published release includes checksums and consistently named assets
The release workflow SHALL publish generated archives to the GitHub Release associated with the pushed tag. Asset names MUST include the project name, the version tag, and the target triple, and the release MUST include a checksum manifest covering the published archives.

#### Scenario: Release assets are uploaded
- **WHEN** the release workflow finishes packaging all targets
- **THEN** the GitHub Release contains the packaged archives plus a checksum manifest for users to verify downloads

#### Scenario: Asset naming follows the release convention
- **WHEN** an archive is generated for a target such as `x86_64-unknown-linux-musl`
- **THEN** the file name includes `gh-download`, the tag version, and the target triple so users can identify the correct binary
