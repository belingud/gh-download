## Why

Directory downloads currently build local output paths from repository metadata paths after stripping the requested root prefix. A compatible or custom API endpoint could return a path with parent-directory segments, an absolute path, or a path outside the requested root, causing the CLI to prepare a local target that does not match the requested directory tree.

## What Changes

- Validate every directory file entry path before joining it to the local directory target.
- Reject directory entries whose metadata path is outside the requested remote root.
- Reject directory-relative output paths that are empty, absolute, contain parent-directory segments, or use platform-specific separator aliases.
- Keep file downloads, proxy behavior, concurrency, overwrite behavior, and output format unchanged.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `github-path-download`: directory downloads must reject unsafe metadata paths before creating local output paths.

## Impact

- Directory download job construction in `src/download.rs`.
- Repository path handling helpers in `src/download/paths.rs`.
- Unit tests for rejected directory metadata paths.
- OpenSpec requirements for recursive directory downloads.
