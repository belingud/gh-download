## MODIFIED Requirements

### Requirement: Directory downloads use bounded concurrent file transfers
After the CLI identifies a remote directory target and enumerates its nested files, it SHALL schedule file downloads so no more than the configured concurrency are in flight at the same time. Each concurrent transfer MUST continue to use the existing raw-download, prefix-proxy, authentication, and stream-to-disk rules for an individual file. If any file transfer fails, the CLI MUST fail the overall directory download command by default. Every file failure MUST be reported with its remote path, resolved local path, and failure reason, and when some files were saved before the failure the CLI MUST also report the partial downloaded and skipped counts. When `--keep-going` is enabled, the CLI SHALL continue scheduling the remaining files after a file failure, collect every failure, and still exit with a non-zero status; the final report MUST list every failed file and the aggregate downloaded, skipped, and failed counts.

#### Scenario: Concurrent directory transfer stays within the configured bound
- **WHEN** a directory download contains more files than the configured concurrency
- **THEN** the CLI downloads the directory files with no more than the configured number of simultaneous file transfers

#### Scenario: A concurrent file transfer fails
- **WHEN** one file download fails during a concurrent directory transfer without `--keep-going`
- **THEN** the CLI reports the directory download as failed instead of reporting overall success

#### Scenario: Failed file is identified
- **WHEN** one file download fails during a concurrent directory transfer
- **THEN** the failure output identifies the failing remote path, its resolved local path, the attempted download strategies, and any files already saved

#### Scenario: Keep-going completes the remaining files
- **WHEN** a user runs a directory download with `--keep-going` and one file fails
- **THEN** the CLI continues downloading the remaining files, lists every failed file, reports aggregate downloaded, skipped, and failed counts, and exits with a non-zero status
