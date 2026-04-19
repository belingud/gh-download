# Capability: code-organization

## Purpose

Define internal maintainability constraints for reorganizing Rust modules without changing the gh-download product contract.

## Requirements

### Requirement: Rust module reorganization preserves product behavior
Rust module reorganization SHALL preserve the existing user-visible behavior and product contracts for CLI parsing, locale selection, terminal output, error classification, proxy handling, GitHub path downloads, filesystem writes, and release packaging.

#### Scenario: Refactor keeps existing behavior contracts
- **WHEN** the large Rust modules are split into smaller modules
- **THEN** the behavior described by the existing `github-path-download`, `prefix-proxy-mode`, `debug-download-flow`, and `binary-release-publishing` capabilities remains unchanged

### Requirement: Public crate exports remain compatible
The crate SHALL keep existing public exports used by current tests and local consumers compatible while internal module paths are reorganized.

#### Scenario: Current public helpers remain importable
- **WHEN** code imports the currently exported CLI, download, error, i18n, and output types or helper functions from the crate root
- **THEN** those imports continue to compile after the module reorganization

### Requirement: Large modules are split by cohesive responsibility
Rust source files SHALL be organized around cohesive responsibilities rather than arbitrary line-count chunks, with orchestration, HTTP transport, path utilities, raw-download strategy handling, CLI resolution, localized help text, and tests separated where practical.

#### Scenario: Download code is reorganized
- **WHEN** `src/download.rs` is refactored
- **THEN** download orchestration remains recognizable while low-level HTTP, path, URL, raw strategy, and model concerns are moved into focused modules

#### Scenario: CLI code is reorganized
- **WHEN** `src/cli.rs` is refactored
- **THEN** clap argument definitions, resolved option construction, environment precedence, local path expansion, localized help text, and tests are separated where practical without changing parsed behavior
