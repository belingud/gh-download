## Context

`gh-download` streams HTTP responses directly into the final local file path. That keeps memory use bounded, but it also means a network or read failure after `File::create` can leave an incomplete file at the path that future default runs treat as an existing file.

The change should stay local to the write path. It must not add CLI options, change proxy behavior, or buffer full downloads in memory.

## Goals / Non-Goals

**Goals:**

- Preserve the existing streaming model while avoiding incomplete final files after failed body copies.
- Keep direct file downloads and per-file directory downloads on the same write path.
- Preserve default skip behavior and explicit overwrite behavior.
- Add focused tests for successful commits and failed copies.

**Non-Goals:**

- Add checksums, resume support, retry policy changes, or a persistent download cache.
- Change user-facing CLI arguments or output wording.
- Guarantee recovery from every possible filesystem failure during the final rename step.

## Decisions

- Use a sibling temporary file for each transfer.
  - Rationale: a temporary file in the same directory keeps the final rename on the same filesystem and avoids cross-device rename errors.
  - Alternative considered: use the system temp directory. That can fail when the destination is on a different filesystem.

- Commit the final target only after the response body copy succeeds.
  - Rationale: failed HTTP body reads then affect only the temporary file, not the resolved final target.
  - Alternative considered: keep writing directly to the final target and clean up on error. That cannot safely distinguish a pre-existing file from a newly truncated one after the target has already been opened.

- Use a small helper around the write/commit flow instead of changing higher-level download orchestration.
  - Rationale: file downloads and directory file downloads already share `stream_download`, so the behavior changes in one place.
  - Alternative considered: add caller-level temporary paths. That would duplicate path and cleanup handling across file and directory flows.

## Risks / Trade-offs

- Temporary files can remain if the process is killed abruptly -> use distinctive sibling names and remove temporary files on ordinary errors.
- On platforms where replacing an existing file via rename is restricted, commit may require a fallback path -> attempt replacement only after the full transfer is available, and restore the previous file if the fallback commit fails.
- The final rename step can still fail due to permissions, locks, or antivirus tools -> return the existing local filesystem error classification without changing CLI behavior.
