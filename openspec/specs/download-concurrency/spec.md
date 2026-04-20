# Capability: download-concurrency

## Purpose

Define the CLI contract and bounded concurrent transfer behavior for directory downloads.

## Requirements

### Requirement: CLI configures bounded directory download concurrency
The CLI SHALL accept a `--concurrency <N>` option and a `-c <N>` short flag for download invocations, where `N` is a positive integer. When the option is omitted, directory downloads SHALL use a default concurrency of `4`. When the remote target resolves to a single file, the CLI SHALL accept the option but continue downloading only that file.

#### Scenario: User sets explicit directory download concurrency
- **WHEN** a user runs `gh-download owner/repo src ./downloads --concurrency 8`
- **THEN** the CLI accepts the invocation and uses `8` as the maximum number of concurrent file downloads for the directory transfer

#### Scenario: User sets explicit directory download concurrency with the short flag
- **WHEN** a user runs `gh-download owner/repo src ./downloads -c 8`
- **THEN** the CLI accepts the invocation and uses `8` as the maximum number of concurrent file downloads for the directory transfer

#### Scenario: User relies on the default directory download concurrency
- **WHEN** a user runs `gh-download owner/repo src ./downloads` without `--concurrency`
- **THEN** the CLI uses a maximum directory download concurrency of `4`

#### Scenario: User provides an invalid concurrency value
- **WHEN** a user runs `gh-download owner/repo src ./downloads --concurrency 0`
- **THEN** the CLI rejects the invocation with a user-visible validation error instead of starting the download

#### Scenario: File downloads remain single-target
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --concurrency 8`
- **THEN** the CLI accepts the invocation and downloads only the resolved file target

### Requirement: Directory downloads use bounded concurrent file transfers
After the CLI identifies a remote directory target and enumerates its nested files, it SHALL schedule file downloads so no more than the configured concurrency are in flight at the same time. Each concurrent transfer MUST continue to use the existing raw-download, prefix-proxy, authentication, and stream-to-disk rules for an individual file. If any file transfer fails, the CLI MUST fail the overall directory download command.

#### Scenario: Concurrent directory transfer stays within the configured bound
- **WHEN** a directory download contains more files than the configured concurrency
- **THEN** the CLI downloads the directory files with no more than the configured number of simultaneous file transfers

#### Scenario: A concurrent file transfer fails
- **WHEN** one file download fails during a concurrent directory transfer
- **THEN** the CLI reports the directory download as failed instead of reporting overall success
