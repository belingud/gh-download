## ADDED Requirements

### Requirement: Repository skill install guidance stays aligned
The repository SHALL maintain `skills/gh-download/SKILL.md` as an agent-facing entrypoint for the CLI. The skill metadata version MUST match the package version in `Cargo.toml`, and its installation guidance MUST include the Homebrew tap command, Cargo installation, and source build path.

#### Scenario: Agent reads the gh-download skill
- **WHEN** an agent opens `skills/gh-download/SKILL.md`
- **THEN** the skill metadata version matches the package version and the installation section includes `brew install belingud/tap/gh-download`, `cargo install gh-download`, and source build guidance
