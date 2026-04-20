## 1. CLI Contract

- [x] 1.1 Add a `--concurrency <N>` CLI option with positive-integer validation and a default value of `4` for directory downloads.
- [x] 1.2 Update localized help text, option resolution, and CLI parsing tests to cover the new concurrency option in both English and Chinese flows.

## 2. Concurrent Directory Downloads

- [x] 2.1 Refactor directory downloads into two phases: recursive file enumeration followed by bounded concurrent file transfer scheduling.
- [x] 2.2 Reuse the existing raw-download, prefix-proxy, authentication, and stream-to-disk logic for each concurrent file transfer without changing single-file download behavior.
- [x] 2.3 Synchronize progress and warning output so concurrent directory downloads remain readable while preserving final download statistics.
- [x] 2.4 Add download tests for default concurrency, explicit concurrency, invalid values, preserved relative paths, and overall failure when any concurrent file transfer fails.

## 3. Docs And Verification

- [x] 3.1 Update `README.md` and `README.zh.md` to document `--concurrency`, its default behavior, and its effect on directory downloads.
- [x] 3.2 Run `just fmt`, `just test`, and `just check` to verify the concurrent download change and related documentation updates.
