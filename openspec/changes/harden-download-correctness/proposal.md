## Why

`gh-download` can currently report success while saving incomplete or misleading content. Directory enumeration uses the Contents API one directory at a time, which silently caps at 1,000 entries, costs one REST request per directory against a 60 request/hour anonymous budget, and keeps requesting a moving branch instead of an immutable revision. When one file fails, the remaining files are dropped without a per-file report. Symlink, submodule, and Git LFS entries are misclassified or written as pointer text because directory listings do not expose their real type. These gaps break the tool's core promise: faithfully save a GitHub repository path to disk.

## What Changes

- Enumerate directories with `GET /repos/{owner}/{repo}/git/trees/{rev}?recursive=1` in one metadata request instead of one Contents API request per directory, with the documented non-recursive per-subtree fallback when the response is `truncated`. This removes the silent 1,000-entry truncation.
- Resolve `--ref` (or the default branch) to an immutable commit SHA before enumerating and downloading, so a moving branch cannot mix revisions inside one saved directory, and expose the resolved SHA in JSON output and debug output.
- Add `--keep-going` for directory transfers. Without it the CLI keeps its current fail-fast contract, but every failure now names the remote path, local path, and attempted strategies, and JSON failures carry partial statistics plus failed entries.
- Make the raw download strategy token-aware: use the documented token-authenticated Contents raw media type as the primary route for private repositories instead of a knowingly-doomed anonymous `download_url` request plus a warning per file; keep prefix-proxy retries for anonymous raw attempts (including when a token is configured, because those attempts carry no credential) and never forward credentials to a proxy.
- Classify non-regular entries explicitly. Directory transfers detect symlinks, submodules, and Git LFS pointer content instead of treating them as regular files: symlinks follow the current transparent behavior by default (`--symlinks auto|link|skip`), submodules are skipped with an actionable warning, and LFS pointers error by default (`--lfs error|media`) instead of silently saving pointer text.
- Improve failure diagnosis: separate network/read failures from local filesystem failures, and report the aggregated per-strategy attempts (direct raw, prefix proxy, raw API) instead of only the last one.
- Extend `--json` coverage and schema: usage/validation failures also produce a JSON document with a stable `code`; failure output includes partial statistics and a `failed` array; success output includes the resolved `ref_sha`.
- Fix local output and path details: disable ANSI color when stdout is not a terminal or `NO_COLOR` is set; treat a trailing path separator as directory intent for file downloads; skip any existing filesystem entry (file or symlink) at the resolved target and fail clearly when that path is a directory; validate metadata-provided file names before joining them with the local target.

**BREAKING**: Git LFS files are no longer silently written as pointer text; the default is a clear error and `--lfs media` opts into the media host best-effort download. Downloading a symlink that points at a directory also stops producing a bogus file containing the target string and is skipped with a warning instead.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `github-path-download`: directory enumeration switches from recursive Contents API calls to a single Git Trees API request with truncation fallback; transfers are pinned to a resolved commit; non-regular entries and LFS objects are classified explicitly; deterministic local path handling covers trailing-separator intent and metadata name validation; status/error output covers TTY-aware color and per-file failure diagnosis.
- `download-concurrency`: directory transfers gain an opt-in `--keep-going` fail-soft mode with aggregated per-file failure reporting and partial statistics; the default remains fail-fast.
- `json-output`: argument/validation failures also emit machine-readable JSON with a stable error code, failure JSON includes partial statistics and failed entries, and success JSON includes the resolved commit.
- `local-write-policy`: skip decisions are based on any existing filesystem entry at the resolved target (not only regular files), and a directory at the resolved file target is a clear error instead of an attempted rename.
- `prefix-proxy-mode`: the raw strategy becomes token-aware, and prefix-proxy retries apply to anonymous raw attempts even when a token is configured; credentials are still never forwarded to a proxy.
- `debug-download-flow`: debug output includes the resolved commit SHA, the tree metadata URL, and the selected enumeration mode.

## Impact

- CLI: new `--keep-going`, `--symlinks <auto|link|skip>`, and `--lfs <error|media>` options; help text and README.md / README.zh.md need matching updates; `skills/gh-download/SKILL.md` needs the new options and a version alignment fix.
- Download core: `src/download.rs`, `src/download/{models,paths,raw,transport}.rs`, new tree/strategy/LFS modules as needed.
- CLI plumbing and output: `src/cli/{types,help,resolve}.rs`, `src/error.rs`, `src/output.rs`, `src/main.rs`, `src/lib.rs`.
- JSON contract: additive fields (`ref_sha`, `failed`, `stats` on failure, stable `error.code`) plus JSON for usage errors.
- Tests: mockito-based tree API scenarios, truncation fallback, commit pinning, token-aware strategy, keep-going aggregation, LFS pointer detection, path intent, and color/TTY behavior.
- No new runtime dependencies are expected; symlink creation uses the standard library and LFS media fetching reuses `reqwest`.
- Release workflow and packaging are unchanged.
