## Context

`gh-download` currently uses a blocking `reqwest` client, detects file-versus-directory targets through the GitHub Contents API, recursively enumerates directory entries, and then downloads directory files one at a time. That keeps the flow simple and preserves the project's explicit proxy and authentication boundaries, but it underutilizes available network bandwidth for directories with many files.

This change is performance-sensitive and cross-cutting. It touches CLI parsing, localized help text, directory scheduling, progress output, and tests. It must keep the current product contract intact: metadata requests stay direct, authenticated requests never go through the public proxy path, and file bytes continue streaming directly to disk instead of being buffered in memory.

## Goals / Non-Goals

**Goals:**
- Add bounded concurrent file downloads for directory targets.
- Expose concurrency as an explicit CLI option with a stable default and validation.
- Preserve the current metadata-first detection flow, relative-path writes, proxy rules, and stream-to-disk behavior.
- Keep English and Chinese help/output semantically aligned.

**Non-Goals:**
- Rewrite the download engine around async Tokio APIs.
- Parallelize GitHub metadata enumeration or replace the Contents API with a different GitHub API.
- Change single-file download semantics.
- Add resumable downloads, retries beyond existing raw-download fallback behavior, or archive download shortcuts.

## Decisions

### 1. Parallelize only the file transfer phase for directory downloads

The runner will keep directory enumeration as a single-threaded recursive Contents API walk, producing the full list of files before any worker pool starts. Only the file transfer phase will run concurrently.

Why:
- This preserves the current metadata contract and avoids mixing recursive API traversal with worker scheduling.
- It keeps unsupported-entry warnings and relative-path calculation in one place.
- It avoids broad behavior changes in the code path that decides whether the remote target is a file or directory.

Alternatives considered:
- Parallelize directory enumeration as well: rejected because it multiplies GitHub metadata requests, complicates warning ordering, and increases the chance of contract drift around proxy/auth behavior.
- Replace recursive enumeration with the Git Trees API: rejected because it changes the existing metadata flow and introduces new edge cases that are not required for this improvement.

### 2. Add an explicit `--concurrency` option with a fixed default of `4`

The CLI will accept `--concurrency <N>` for download invocations. The value must be a positive integer. When omitted, directory downloads will use `4` workers. Single-file downloads will accept the option but continue using a single transfer.

Why:
- A user-visible performance feature needs a predictable escape hatch when a repository, network, or host environment behaves poorly.
- A fixed default is easier to document and test than deriving from CPU count or network heuristics.
- Keeping `1` as a valid value provides an explicit sequential mode without inventing a separate flag.

Alternatives considered:
- Use an unconfigurable worker count: rejected because it makes troubleshooting and low-resource environments harder.
- Derive the default from available CPUs: rejected because it is less predictable across machines and does not map cleanly to network-bound behavior.
- Add `--threads` instead: rejected because the product contract is about concurrency, not the implementation detail of threads.

### 3. Use a small scoped worker pool on top of the existing blocking transport

Implementation should stay within the current blocking architecture. After directory enumeration, the runner will hand file download jobs to a bounded pool of worker threads. Each worker will reuse the same download logic, including raw URL fallback, prefix-proxy rules, and direct streaming to a distinct local file path.

Why:
- It avoids a whole-program async rewrite and keeps most transport code reusable.
- `reqwest::blocking::Client` already matches the current code path and can be reused safely by worker threads.
- Each file already maps to a unique local target, so concurrent writes do not require redesigning the on-disk layout.

Alternatives considered:
- Switch to `tokio` + async `reqwest`: rejected because it would turn a focused performance change into a large architectural refactor.
- Use a parallel iterator crate such as Rayon for the entire flow: rejected because a dedicated worker pool gives tighter control over error handling and output synchronization.

### 4. Serialize user-visible output while allowing transfer order to vary

Concurrent directory downloads will keep the current summary structure, but per-file progress lines will no longer imply repository order. Output calls that can originate from workers should be synchronized so lines remain readable in both languages. The completion summary will continue to report the final saved path and aggregate counts.

Why:
- Readability is part of the product surface, and unsynchronized worker prints would make concurrent logs noisy and misleading.
- Allowing output order to vary keeps the implementation simple without promising deterministic transfer order.

Alternatives considered:
- Suppress per-file progress during concurrent downloads: rejected because it would reduce visibility compared with the current CLI behavior.
- Promise repository-order output: rejected because that would require extra coordination that does not improve the saved result.

### 5. Fail the overall command on any file-transfer error and leave completed files in place

If any worker fails to download a file, the overall directory download will fail. Files already written successfully will remain on disk, matching the current practical behavior that partial progress is not rolled back.

Why:
- Silent partial success would be misleading for a directory download command.
- Rolling back completed files adds complexity and risks deleting user data unexpectedly.

Alternatives considered:
- Best-effort completion with a warning: rejected because the current CLI contract treats download failures as command failures.
- Delete all completed files on error: rejected because cleanup would be error-prone and surprising.

## Risks / Trade-offs

- [Concurrent transfers may trigger GitHub rate limits sooner for anonymous users] -> Keep the worker count bounded, preserve the existing proxy/auth rules, and let users reduce concurrency explicitly.
- [Multiple workers can make progress logs harder to follow] -> Synchronize output writes and keep messages short.
- [A worker-pool refactor may introduce subtle error-accounting bugs] -> Add tests for default concurrency, explicit concurrency, invalid values, mixed success/failure, and preserved relative paths.
- [Partial files may remain after a failed concurrent run] -> Document fail-on-error behavior and keep `--concurrency 1` available for troubleshooting.

## Migration Plan

1. Add the new capability and delta specs for concurrency configuration and concurrent directory downloads.
2. Extend CLI parsing and localized help text with `--concurrency`.
3. Refactor directory download execution into two phases: enumerate files, then schedule bounded concurrent transfers.
4. Add tests covering CLI validation, default behavior, bounded concurrency, unchanged proxy boundaries, and directory path preservation.
5. Update `README.md` and `README.zh.md` with the new option and any output examples that mention directory downloads.

Rollback strategy:
- If the concurrent transfer path proves unstable during implementation, fall back to the current sequential execution path and keep the change unmerged until the worker-pool logic is reliable.

## Open Questions

- None currently.
