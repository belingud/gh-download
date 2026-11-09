## Context

See proposal.md - Why. The current download core makes one Contents API request per directory, follows a branch name on every request, downloads every listed entry as a regular file, and maps body-copy failures to local filesystem errors. Existing contracts that must survive: streamed writes to a temporary file followed by an atomic commit, default skip-existing behavior, explicit `--overwrite`, fail-fast directory transfers by default, direct-only metadata requests, no credential forwarding to public proxies, exit status `0`/`1`/`2`, and additive JSON evolution.

## Goals / Non-Goals

**Goals:**

- Enumerate a directory with a constant number of metadata requests and never silently drop entries above the Contents API directory limit.
- Bind one invocation to one commit SHA so detection, enumeration, and file bytes describe the same revision.
- Make private-repository downloads use the documented token-authenticated route without per-file doomed anonymous requests or misleading warnings.
- Report directory-transfer failures per file with enough context to act, and offer an opt-in fail-soft mode.
- Handle symlinks, submodules, and Git LFS objects explicitly instead of writing misleading files.
- Make JSON output usable by automation for usage failures and partial transfers.
- Fix local-target intent, skip, and color behavior without changing default exit semantics.

**Non-Goals:**

- Archive/tarball transport (`tarball`/`zipball` streaming extraction). It would change proxy boundaries and add compression dependencies; it remains the natural follow-up for very large private trees.
- Bounded retry/backoff policy, rate-limit preflight, progress bars, resume/manifest, include/exclude filters, optional positional arguments, explicit HTTP proxy, release asset downloads.
- Recursing into submodule repositories or implementing the official Git LFS batch API.
- Changing the configuration-file key set or release workflow.

## Decisions

### 1. Enumerate directories with one recursive tree request

Use `GET {api_base}/repos/{owner}/{repo}/git/trees/{commit_sha}?recursive=1` for directory transfers. Tree entries carry `path`, `type` (`blob`/`tree`/`commit`) and `mode` (`100644`, `100755`, `120000`, `160000`), which is exactly the classification the Contents listing lacks. When `truncated` is `true`, fall back to the documented workaround: fetch the root tree non-recursively, then request each subtree recursively, recursing further only if a subtree response is also truncated.

The Contents API stays in use for the initial file-versus-directory detection of the requested path (one request with `?ref={commit_sha}`); downloading a single file must not fetch the whole recursive tree (up to 7 MB).

- Alternative: keep Contents recursion. Rejected: one request per directory against a 60/hour anonymous budget and a silent 1,000-entry cap.
- Alternative: download an archive and extract. Deferred to a follow-up because it changes the raw-download/proxy contract, needs tar/gzip crates, and GitHub archives exclude LFS objects by default.

### 2. Resolve the transfer to one commit SHA

Resolution order: a 40-hex `--ref` is used directly; otherwise `GET /repos/{owner}/{repo}/commits/{ref}` returns the commit SHA; without `--ref`, fetch `GET /repos/{owner}/{repo}` once for `default_branch` and resolve that branch. All later requests use the SHA: Contents detection (`?ref={sha}`), tree enumeration, constructed raw URLs, and JSON/debug diagnostics.

- Rationale: a branch that moves between enumeration and file downloads currently produces mixed revisions; pinning also gives automation a reproducible identifier.
- Alternative: resolve lazily only for directory transfers. Rejected: single-file JSON would lack `ref_sha`, and debug output would be inconsistent.
- Cost is bounded: at most two extra metadata requests per invocation, never per file.

### 3. Token-aware raw strategy and visibility detection

Select the primary byte-transfer route from token presence, repository visibility, and API base:

| Situation | Primary route | Prefix proxy modes |
|---|---|---|
| No token | Constructed raw URL for public GitHub; proxy modes apply | Yes |
| Token + public repository | Constructed raw URL (anonymous, no token forwarded); API raw fallback | Yes |
| Token + private repository | Contents raw media type with token | Not applicable (no anonymous raw URL attempt) |
| Custom `--api-base` (GHE) | API-provided raw download URL, Contents raw media fallback on the same base | Not applicable |

When a token is present, fetch `GET /repos/{owner}/{repo}` once to learn `private` (the same call supplies `default_branch`, so it is not wasted). Constructed raw URLs are only used for the default public GitHub API base, where the host is known: `https://raw.githubusercontent.com/{owner}/{repo}/{sha}/{encoded path}`.

Prefix-proxy retries apply to anonymous raw URL attempts even when a token is configured, because those attempts carry no credentials; the token is never attached to a proxied URL. For private repositories, `prefer`/`fallback` do not send the private path to the prefix proxy at all.

- Alternative: attach the token to the raw host (works in practice but is undocumented). Rejected as the primary route; it can be revisited as an opt-in fallback.
- Alternative: learn visibility from the first raw 404 (sticky strategy). Rejected: it still burns one request and emits exactly the warning this change removes.

### 4. Fail-fast by default, aggregate reporting always, `--keep-going` opt-in

Keep the existing bounded batch scheduler. Change the result plumbing so a batch returns per-item outcomes instead of collapsing to the first error, and introduce one aggregated failure type carrying: the failed entries (remote path, local path, strategy attempts, reason), partial statistics, and whether scheduling stopped early.

- Default behavior stays fail-fast: after a batch with a failure, remaining batches are not scheduled, but the error now identifies the failing file, the attempted strategies, and how many files were already saved.
- `--keep-going` schedules every batch, collects all failures, and still returns the aggregated failure so the process exits `1`.
- Exit codes remain `0` success, `1` transfer/validation failure, `2` CLI usage error. No new exit code is introduced for partial success.

JSON failure payload gains optional `stats` and `failed` alongside the existing `error` object; the existing fields keep their meaning.

- Alternative: continue by default. Rejected: it is a user-visible contract change for scripts that rely on stopping early; the opt-in flag keeps the default contract while making failures diagnosable.

### 5. Classify non-regular entries from tree modes; detect LFS after the body copy

Tree mode is the classification source of truth: `120000` is a symlink, `160000` a submodule, `100644`/`100755` regular blobs, `040000` a directory.

- `--symlinks auto` (default): for a symlink entry, fetch the direct Contents metadata for that path at the SHA. If GitHub resolves it to a regular file (the common in-repo link), schedule it like a normal file at the symlink's relative path, preserving current effective behavior. If it remains a symlink (target is a directory/submodule), warn and skip. `skip` warns and skips all symlinks. `link` resolves the symlink target through the git blob raw media type (`GET /git/blobs/{sha}` with a raw `Accept`) and creates a local symlink; platforms without symlink support fail that entry with an actionable message.
- Submodules are skipped with a warning that includes the submodule URL from direct Contents metadata and the pinned commit recorded in the tree.
- LFS detection happens after a transfer body is copied into the temporary file: read a small prefix (≤256 bytes) and match the `version https://git-lfs.github.com/spec/v1` pointer magic. `--lfs error` (default) deletes the temporary file and fails the file with an LFS-specific error. `--lfs media` retries only the object through `https://media.githubusercontent.com/media/{owner}/{repo}/{sha}/{path}` (public GitHub host only, best effort) and falls back to the error behavior when the host fails.

- Rationale: trusting a Contents listing's `type` is unsafe because submodules appear as `type: file, size: 0, download_url: null` and symlinks appear as `type: file, size: <target length>`; a null `download_url` was the only workable signal before tree metadata.
- Alternative: always materialize symlinks. Rejected: changes default behavior for repositories that use symlinks as content aliases.
- Alternative: implement the official LFS batch API. Deferred: bigger scope and auth semantics; `--lfs media` is a bounded best-effort bridge, and the default is a clear error rather than corrupt output.

### 6. Separate network failures from local write failures

Replace `io::copy` in the transfer path with an explicit read/write loop over the response reader: read errors map to a new network-oriented error, while write errors keep the existing local-I/O error and its permission/disk suggestions. Preserve the temporary-file commit flow and cleanup rules unchanged.

Download strategy fallbacks collect attempt records (strategy label, redacted error summary). The final error carries that list, so the user sees every attempted route instead of only the last one.

- Rationale: most current body-copy failures are network stalls/resets misreported as disk problems, and the final error currently hides the root cause behind the last fallback.

### 7. JSON covers usage failures and partial transfers

`--json` is also detected by a pre-parse argument scan (same technique already used for `--config` and `--lang`). When argument parsing fails and `--json` was requested, render one JSON failure document with `error.code = "usage"` on stdout and keep exit status `2`.

`UserFacingError` gains a stable `code` (`usage`, `config`, `authentication`, `not_found`, `network`, `io`, `lfs`, `partial_transfer`, `unexpected_response`). JSON shapes stay additive:

- success: `success`, `saved_path`, `ref_sha`, `stats` (downloaded, skipped existing, skipped unsupported entries, skipped symlinks, skipped submodules)
- failure: `success`, `error { code, title, reason, suggestions }`, optional `stats`, optional `failed [{ path, local_path, reason }]`

### 8. Local target intent, skip rules, and color

- Capture trailing-separator intent from the raw argument before lexical normalization (the `Path` value drops it). Carry it as an internal `local_target_dir_hint`. For file downloads, a hint creates the directory and writes the validated file name inside it; directory-download target semantics are unchanged.
- Skip checks use "any entry exists at the resolved target" (`symlink_metadata`/`try_exists`). A directory at the resolved file target is a dedicated error for both default and overwrite modes; symlinks are skipped/replaced per `--overwrite`.
- Validate metadata-provided file names (single normal component, no separators, no `.`/`..`, not absolute, no NUL) before joining. On rejection, fall back to the validated remote path file name; fail when neither is safe. Also restrict download URLs to `http`/`https`.
- Color is enabled only when stdout is a terminal, `NO_COLOR` is unset, and `--no-color` is absent (`std::io::IsTerminal`). JSON mode already suppresses human output.

### 9. Module shape

Add `src/download/tree.rs` (tree response models, recursive enumeration, truncation fallback), `src/download/strategy.rs` (route selection, raw URL construction, attempt records), and `src/download/lfs.rs` (pointer detection, media fetch). Extend the data model with `sha`, `size`, `mode`, `target`, and `submodule_git_url` instead of dropping unknown fields. Keep `download.rs` as orchestration only.

## Risks / Trade-offs

- [Trees API adds constant metadata requests and a repo-visibility request] → Accept the bounded cost; debug output explains every request.
- [Very large private directory transfers still burn one API request per file] → Document the 5,000/hour authenticated ceiling; archive transport is the named follow-up.
- [Recursive tree responses can be huge (7 MB cap / 100k entries)] → Stream JSON parsing is out of scope; sizes stay bounded, and truncated responses fall back to per-subtree requests.
- [LFS media host is undocumented and may change] → Best-effort only behind `--lfs media`; default is a clear error; failures fall back to the actionable LFS error.
- [Symlink `link` behavior differs by platform] → Fail per entry on unsupported platforms; document in README and help.
- [More warning categories could make output noisy] → Warnings are per category per file only when actionable, and skipped counts are aggregated in completion/JSON.
- [Commit pinning can make `--ref` resolution fail earlier for unusual refs] → Surface a specific error identifying the requested ref; existing `--ref` support (branch/tag/SHA) is unchanged.
- [GHE deployments expose raw content on the same base but not always a reliable raw hostname] → Keep the API-provided download URL as the primary route and use the Contents raw media type on the configured base as fallback; metadata is never routed through prefix proxies.

## Migration Plan

No persisted state or data migration. Ship the whole change in one release: implementation, tests, README.md/README.zh.md, help text, and `skills/gh-download/SKILL.md` (including the existing version mismatch fix). Rollback is a normal release revert; no on-disk compatibility is affected. The only intentional behavior changes are called out in the proposal (`**BREAKING**`): LFS pointer files fail by default, and directory-symlink entries are skipped instead of written as target text.

## Open Questions

- Should `--lfs media` also attempt a token-authenticated retrieval for private repositories? Deferred: current scope keeps LFS media public-GitHub only and errors otherwise.
- Should Windows developer-mode symlink creation be attempted for `--symlinks link`? Deferred: current scope returns an actionable per-entry error on platforms without symlink support.
