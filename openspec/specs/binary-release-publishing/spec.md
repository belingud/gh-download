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
The release workflow SHALL package Unix targets as `.tar.gz` archives and Windows targets as `.zip` archives. Each archive MUST contain the platform-specific `gh-download` executable, `README.md`, `README.zh.md`, and `LICENSE`.

#### Scenario: Unix asset is prepared
- **WHEN** a Linux or macOS target is packaged
- **THEN** the workflow creates a `.tar.gz` archive containing the executable, `README.md`, `README.zh.md`, and `LICENSE`

#### Scenario: Windows asset is prepared
- **WHEN** the Windows target is packaged
- **THEN** the workflow creates a `.zip` archive containing `gh-download.exe`, `README.md`, `README.zh.md`, and `LICENSE`

### Requirement: Release workflow verifies archives before upload
The release workflow SHALL inspect each generated archive before uploading it as a build artifact. The verification MUST fail the build job when the archive does not contain the platform-specific executable, `README.md`, `README.zh.md`, or `LICENSE`. The workflow MUST run the packaged executable with `--version` on runner platforms that can execute the target binary, and it MUST perform this verification before `actions/upload-artifact`.

#### Scenario: Unix archive is verified before upload
- **WHEN** a Linux or macOS target creates a `.tar.gz` archive
- **THEN** the workflow verifies that the archive contains `gh-download`, `README.md`, `README.zh.md`, and `LICENSE` before uploading the artifact

#### Scenario: Windows archive is verified before upload
- **WHEN** the Windows target creates a `.zip` archive
- **THEN** the workflow verifies that the archive contains `gh-download.exe`, `README.md`, `README.zh.md`, and `LICENSE` before uploading the artifact

#### Scenario: Native runner binary version is checked
- **WHEN** a packaged binary can run on the current GitHub Actions runner
- **THEN** the workflow runs the packaged binary with `--version` before uploading the artifact

### Requirement: Published release includes checksums and consistently named assets
The release workflow SHALL publish generated archives to the GitHub Release associated with the pushed tag. Asset names MUST include the project name, the version tag, and the target triple, and the release MUST include a checksum manifest covering the published archives.

#### Scenario: Release assets are uploaded
- **WHEN** the release workflow finishes packaging all targets
- **THEN** the GitHub Release contains the packaged archives plus a checksum manifest for users to verify downloads

#### Scenario: Asset naming follows the release convention
- **WHEN** an archive is generated for a target such as `x86_64-unknown-linux-musl`
- **THEN** the file name includes `gh-download`, the tag version, and the target triple so users can identify the correct binary

### Requirement: Published release includes generated release notes
The release workflow SHALL generate a Markdown release notes document and publish it as the GitHub Release body. The release body MUST identify the version tag, describe supported release assets, include installation guidance in English and Chinese for Homebrew, Cargo, and prebuilt archives, include checksum verification guidance, and include a generated changes section for the tagged release.

#### Scenario: Release notes are generated before publishing
- **WHEN** the release job has downloaded packaged artifacts and generated `checksums.txt`
- **THEN** the workflow writes a release notes Markdown file that is used as the GitHub Release body

#### Scenario: Release notes describe assets and verification
- **WHEN** a tag-triggered release is published
- **THEN** the GitHub Release body names the supported platform assets and explains how to verify downloads with `checksums.txt`

#### Scenario: Release notes include multilingual installation guidance and changes
- **WHEN** a user views the GitHub Release for a tag
- **THEN** the body includes concise English and Chinese installation guidance for Homebrew, Cargo, and prebuilt archives plus a generated changes section for the tagged release
