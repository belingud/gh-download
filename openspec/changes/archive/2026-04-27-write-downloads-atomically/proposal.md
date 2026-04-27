## Why

`stream_download` currently creates or truncates the final target before copying the HTTP response body. If the transfer fails after that point, the final path can contain an incomplete file, and a later default run may skip it as an existing file.

## What Changes

- Write downloaded bytes to a sibling temporary file before committing the final target.
- Commit the temporary file to the final target only after the HTTP body has been copied successfully.
- Remove temporary files after failed transfers where possible.
- Keep the current CLI arguments, default skip behavior, overwrite behavior, and streaming download model.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `github-path-download`: file downloads still stream to disk, but the final target must not be created or truncated before the transfer body has completed successfully.
- `local-write-policy`: overwrite mode still replaces existing files, but replacement should be committed only after the replacement transfer has completed successfully.

## Impact

- `src/download/transport.rs` download write path.
- Unit tests for failed stream copies and successful commits.
- OpenSpec requirements for download path handling and overwrite behavior.
