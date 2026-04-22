## MODIFIED Requirements

### Requirement: CLI configures bounded directory download concurrency
The CLI SHALL accept a `--concurrency <N>` option and a `-c <N>` short flag for download invocations, where `N` is a positive integer. The CLI SHALL also accept `concurrency` from the configuration file as a persisted default for download invocations. When neither CLI arguments nor configuration file provide a concurrency value, directory downloads SHALL use a default concurrency of `4`. When the remote target resolves to a single file, the CLI SHALL accept the option or configuration value but continue downloading only that file.

#### Scenario: User sets explicit directory download concurrency
- **WHEN** a user runs `gh-download owner/repo src ./downloads --concurrency 8`
- **THEN** the CLI accepts the invocation and uses `8` as the maximum number of concurrent file downloads for the directory transfer

#### Scenario: User sets explicit directory download concurrency with the short flag
- **WHEN** a user runs `gh-download owner/repo src ./downloads -c 8`
- **THEN** the CLI accepts the invocation and uses `8` as the maximum number of concurrent file downloads for the directory transfer

#### Scenario: Configuration file provides directory download concurrency
- **WHEN** a user runs `gh-download owner/repo src ./downloads` and the active configuration file contains `concurrency = 6`
- **THEN** the CLI uses `6` as the maximum number of concurrent file downloads for the directory transfer

#### Scenario: User relies on the default directory download concurrency
- **WHEN** a user runs `gh-download owner/repo src ./downloads` without `--concurrency` and without a configuration-file `concurrency`
- **THEN** the CLI uses a maximum directory download concurrency of `4`

#### Scenario: User provides an invalid concurrency value
- **WHEN** a user runs `gh-download owner/repo src ./downloads --concurrency 0`
- **THEN** the CLI rejects the invocation with a user-visible validation error instead of starting the download

#### Scenario: Configuration file provides an invalid concurrency value
- **WHEN** a user runs `gh-download owner/repo src ./downloads` and the active configuration file contains `concurrency = 0`
- **THEN** the CLI rejects the invocation with a user-visible configuration error instead of starting the download

#### Scenario: File downloads remain single-target
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --concurrency 8`
- **THEN** the CLI accepts the invocation and downloads only the resolved file target
