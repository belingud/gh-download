## ADDED Requirements

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
