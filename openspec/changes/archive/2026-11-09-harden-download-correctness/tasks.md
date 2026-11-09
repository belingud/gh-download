## 1. Commit pinning and metadata resolution

- [x] 1.1 Add ref-to-commit resolution that reuses a 40-hex `--ref` directly, resolves other refs through `GET /repos/{owner}/{repo}/commits/{ref}`, and resolves the default branch to a commit when `--ref` is absent; verify with mockito tests for branch, tag, SHA, and missing-ref cases.
- [x] 1.2 Read repository metadata once when a token is present or `--ref` is absent to obtain `private` and `default_branch`; verify the request count stays constant per invocation in a mockito test.
- [x] 1.3 Thread the resolved commit SHA through detection, enumeration, downloads, debug output, and the `RunOutcome`; verify `--debug` prints the resolved SHA and that a mock branch move cannot change downloaded bytes.
- [x] 1.4 Fail with a user-visible error identifying the requested ref when resolution fails; verify the error text and exit status in a CLI integration test.

## 2. Directory enumeration via Git Trees API

- [x] 2.1 Add tree response models (path, type, mode, sha, size) and a recursive enumerator using `?recursive=1`; verify nested directory fixtures enumerate with one tree request per invocation.
- [x] 2.2 Implement the truncated-response fallback to non-recursive root and per-subtree tree requests; verify with a mockito fixture that sets `truncated: true` and asserts the fallback path resolves every file.
- [x] 2.3 Replace the per-directory Contents enumeration in the directory flow while keeping the Contents request for file-versus-directory detection; verify a directory fixture with more than 1,000 entries downloads all files in a new regression test.
- [x] 2.4 Keep existing relative-path validation and concurrency scheduling over the tree result; verify the existing unsafe-metadata-path tests still pass and add a tree-based traversal fixture.
- [x] 2.5 Emit tree metadata URL and enumeration mode in debug output; verify with a debug-mode subprocess test.

## 3. Token-aware raw strategy

- [x] 3.1 Add a strategy selector that chooses Contents raw media for private repositories with a token and for custom `--api-base`, and constructed raw URLs for the public no-token/public-token cases; verify each matrix row with mockito request assertions.
- [x] 3.2 Construct SHA-pinned raw URLs with per-segment encoding for the default public GitHub host; verify URL output for paths with spaces and unicode.
- [x] 3.3 Allow prefix-proxy retries for anonymous raw URL attempts when a token is configured without attaching the token to the proxied URL; verify the proxied request has no `Authorization` header and that the retry happens.
- [x] 3.4 Remove the per-file anonymous raw attempt and warning for private repositories; verify a private-repository fixture records exactly one token-authenticated request per file with no warning output.
- [x] 3.5 Preserve raw-API fallback for public repositories whose raw URL attempts fail; verify existing fallback tests still pass.

## 4. Failure aggregation and `--keep-going`

- [x] 4.1 Introduce an aggregated transfer failure type carrying failed entries (remote path, local path, strategy attempts), partial statistics, and whether scheduling stopped early; verify serialization/debug output in a unit test.
- [x] 4.2 Collect per-attempt strategy errors during a file fallback chain and include them in the final failure; verify a fixture with a direct 503 and a raw-API 404 reports both attempts.
- [x] 4.3 Return per-item outcomes from the bounded scheduler instead of the first error while keeping the fail-fast default; verify the remaining batches are not scheduled.
- [x] 4.4 Add `--keep-going` and continue scheduling all batches on failure, aggregating every failed entry and still returning a non-zero result; verify with a three-file fixture where the middle file fails.
- [x] 4.5 Report partial saved counts and failing file identity in human error output; verify the wording in an output unit test.
- [x] 4.6 Keep exit statuses unchanged (`0` success, `1` transfer failure, `2` usage error); verify in CLI integration tests.

## 5. Non-regular entries and Git LFS

- [x] 5.1 Classify tree entries by mode (regular blob, symlink, submodule) and pass the classification into the download job; verify mixed-tree fixtures produce the expected kind summary.
- [x] 5.2 Implement `--symlinks auto|link|skip` with `auto` as default: follow symlinks that resolve to regular files, skip directory/submodule symlinks with warnings, and materialize links via git blob raw media under `link`; verify all three modes with mockito fixtures.
- [x] 5.3 Skip submodule entries with a warning that includes the submodule URL and pinned commit; verify the warning text and skipped counts.
- [x] 5.4 Detect Git LFS pointer content from the temporary file prefix and fail with an LFS-specific error under the default `--lfs error`; verify pointer text is never committed as the downloaded file.
- [x] 5.5 Implement `--lfs media` retrieval from the GitHub media host for the public API base and fall back to the actionable error otherwise; verify success and failure paths with mockito.
- [x] 5.6 Reject single-file invocations whose requested path resolves to a submodule or non-file symlink with an actionable error; verify both fixtures.
- [x] 5.7 Add skipped symlink/submodule counters to statistics and completion output; verify counts in human and JSON output tests.

## 6. Local target and output behavior

- [x] 6.1 Capture trailing-separator directory intent before path normalization and apply it to file downloads; verify `gh-download owner/repo README.md ./downloads/` creates the directory and writes the file inside it.
- [x] 6.2 Replace the `is_file()` skip check with an existence-based check and add a dedicated error when the resolved target is a directory; verify default and `--overwrite` behavior for file, symlink, and directory targets.
- [x] 6.3 Validate metadata-provided file names before joining and fall back to a safe remote path name; verify `..`/separator/absolute fixtures cannot escape the target directory.
- [x] 6.4 Restrict download URL schemes to `http`/`https`; verify a non-HTTP `download_url` fixture is rejected.
- [x] 6.5 Disable ANSI color for non-terminal stdout, `NO_COLOR`, and keep `--no-color` authoritative; verify with piped subprocess output and a `NO_COLOR` environment fixture.

## 7. JSON contract and usage errors

- [x] 7.1 Pre-scan arguments for `--json`/`--lang`/`--no-color` and emit a JSON failure document for clap validation errors while keeping exit status `2`; verify missing positional arguments and invalid `--concurrency` values.
- [x] 7.2 Add a stable `error.code` to classified errors and the JSON failure payload; verify codes for usage, authentication, not-found, network, IO, and LFS cases.
- [x] 7.3 Add `ref_sha` to the JSON success payload and skipped symlink/submodule counters to `stats`; verify the documented success shape.
- [x] 7.4 Add optional partial `stats` and `failed` entries to the JSON failure payload for fail-fast and `--keep-going` directory transfers; verify both shapes with fixture downloads.
- [x] 7.5 Keep human-readable output suppressed on stdout in JSON mode in all new paths; verify with the existing JSON/debug subprocess tests extended to failure and usage cases.

## 8. Documentation

- [x] 8.1 Update `README.md` and `README.zh.md` with the new options, pinned-commit behavior, token-aware strategy, LFS/symlink behavior, JSON schema additions, and the GHE raw-media limitation; verify both documents describe the same behavior.
- [x] 8.2 Update `skills/gh-download/SKILL.md` with the new options and behavior, and fix the skill version to match `Cargo.toml`; verify the version line matches the package version.
- [x] 8.3 Update localized help text for `--keep-going`, `--symlinks`, and `--lfs`; verify the rendered help contains the new options in English and Chinese.

## 9. Verification

- [x] 9.1 Run `openspec validate harden-download-correctness --strict` and `openspec validate --all`; verify the change and all specs validate.
- [x] 9.2 Run `just fmt`, `just test`, and `just check`; verify formatting, unit tests, and integration tests pass.
- [x] 9.3 Run manual smoke tests with the final built binary: a 1,001-entry directory fixture enumerates and downloads completely, and real-repository scenarios (root directory, pinned ref, symlink, LFS media) pass.
- [x] 9.4 Run a manual `--json --keep-going` smoke test with an intentionally failing file; verify the failure JSON contains the stable code, partial stats, and failed entry.
