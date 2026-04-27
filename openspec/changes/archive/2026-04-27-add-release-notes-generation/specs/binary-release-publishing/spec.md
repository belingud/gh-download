## ADDED Requirements

### Requirement: Published release includes generated release notes
The release workflow SHALL generate a Markdown release notes document and publish it as the GitHub Release body. The release body MUST identify the version tag, describe supported release assets, include installation guidance in English and Chinese, include checksum verification guidance, and include a generated changes section for the tagged release.

#### Scenario: Release notes are generated before publishing
- **WHEN** the release job has downloaded packaged artifacts and generated `checksums.txt`
- **THEN** the workflow writes a release notes Markdown file that is used as the GitHub Release body

#### Scenario: Release notes describe assets and verification
- **WHEN** a tag-triggered release is published
- **THEN** the GitHub Release body names the supported platform assets and explains how to verify downloads with `checksums.txt`

#### Scenario: Release notes include multilingual installation guidance and changes
- **WHEN** a user views the GitHub Release for a tag
- **THEN** the body includes concise English and Chinese installation guidance plus a generated changes section for the tagged release
