## Context

`gh-download` currently resolves the final local output path and then writes file bytes with `File::create`, which replaces any existing file at that path. That behavior is simple but unsafe for repeated runs, especially when the CLI is used in automation or for incremental directory downloads. The requested change is user-visible and cross-cutting because it affects CLI flags, download execution, progress/output wording, completion accounting, and documentation.

This repository already treats CLI behavior and download semantics as part of the product contract. Any change here must preserve the existing metadata-first GitHub detection flow, streaming writes, directory structure handling, and proxy/authentication boundaries.

## Goals / Non-Goals

**Goals:**
- Make existing local files safe by default by skipping them unless the user explicitly requests overwrite behavior.
- Add an explicit `--overwrite` flag for users who want the previous replacement behavior.
- Apply the same conflict policy to both direct file downloads and per-file writes inside directory downloads.
- Make skipped-versus-written outcomes visible in CLI output and final stats.

**Non-Goals:**
- Add resumable or partial-file resume behavior.
- Change how local target paths are resolved for files versus directories.
- Add hash-based comparison, mtime comparison, or content-diff logic before deciding to skip.
- Change raw download, proxy fallback, or GitHub metadata request behavior.

## Decisions

### 1. Default to skip-on-conflict and expose `--overwrite` as the explicit opt-in

When the final local file target already exists, the CLI will skip writing that file by default. Passing `--overwrite` switches the behavior back to replacing the existing file.

Why:
- This aligns the default behavior with safer automation expectations.
- It gives users a simple, explicit escape hatch without introducing multiple conflict modes immediately.
- It keeps the product contract small: only two states matter for v0.4.0, safe default and explicit overwrite.

Alternatives considered:
- Keep overwrite as the default and add `--skip-existing`: rejected because the user explicitly wants safer default behavior.
- Add a larger policy matrix (`skip`, `overwrite`, `fail`): rejected for now because it expands the contract before the simpler policy is validated in practice.

### 2. Resolve path conflicts at the final file target, not at the parent directory level

The conflict check will happen after the CLI has already resolved the same final local file path it would otherwise write. For file downloads, that means after handling “target is an existing directory” logic. For directory downloads, that means after calculating each relative file path inside the output directory.

Why:
- It preserves the current local path rules and only changes what happens when the resolved file already exists.
- It avoids introducing separate conflict semantics for file mode and directory mode.

Alternatives considered:
- Decide based on the raw user-provided path before file target resolution: rejected because it would create inconsistent behavior between direct file targets and existing destination directories.

### 3. Treat skipped existing files as successful command outcomes, not errors

Skipping an existing file will not fail the command. The runner will track how many files were downloaded, how many were skipped because they already existed, and how many unsupported repository entries were skipped.

Why:
- A safe default should not force users to handle errors for normal repeated runs.
- Directory downloads often need best-effort incremental behavior, and failing on the first existing file would be noisy and unhelpful.

Alternatives considered:
- Fail on the first existing file unless `--overwrite` is set: rejected because it makes repeated runs brittle and defeats the safe-default goal.

### 4. Surface overwrite policy in concise progress and completion output

When a file is skipped because it already exists, the CLI should print a concise message identifying the file path and the reason. The completion summary should include skipped-existing counts separately from skipped unsupported entries so users can tell whether files were preserved or ignored for repository-shape reasons.

Why:
- Once overwrite stops being implicit, the user needs clear feedback about what the CLI did not write.
- Separate accounting avoids overloading the existing “skipped entries” count, which currently refers to unsupported GitHub metadata entry types.

Alternatives considered:
- Only report skipped-existing files in debug mode: rejected because the behavior change is user-visible and should be visible in normal mode.

## Risks / Trade-offs

- [Existing automation may rely on implicit overwrite behavior] -> Mark the change as breaking in the proposal, document `--overwrite` clearly, and keep the flag name obvious.
- [Large directory downloads may print many skip lines] -> Keep messages short and aggregate skipped-existing counts in the final summary.
- [Users may expect “skip existing” to compare file contents] -> Document that v0.4.0 only checks path existence, not content equality.
- [Changing completion stats could affect downstream documentation/examples] -> Update README examples and related specs together with the implementation.

## Migration Plan

1. Add specs for local write conflict policy and update the core download spec to mention `--overwrite`.
2. Extend CLI parsing/help to support `--overwrite`.
3. Refactor file write logic so it can check the resolved local target and either skip or overwrite before streaming content.
4. Update progress and completion output to report skipped-existing files separately.
5. Add tests for direct file downloads, directory downloads, existing destination files, and explicit overwrite behavior.
6. Update English and Chinese docs to explain the new default and the migration path for users who need replacement behavior.

Rollback strategy:
- If the new default causes unacceptable compatibility issues during implementation or testing, revert to the previous overwrite behavior before release and revisit the change with a more explicit migration plan.

## Open Questions

- None currently.
