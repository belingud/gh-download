## Why

The release workflow packages archives before publishing, but it does not currently verify archive contents before upload. A packaging regression could publish an archive that misses the binary, README files, or license.

## What Changes

- Add a release workflow validation step after packaging and before artifact upload.
- Verify each generated `.tar.gz` or `.zip` archive contains the platform binary, `README.md`, `README.zh.md`, and `LICENSE`.
- Run `--version` against the packaged binary on runner platforms that can execute the target binary.
- Keep asset names, platform matrix, checksum generation, and release publishing behavior unchanged.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `binary-release-publishing`: release archives must be validated before they are uploaded as build artifacts.

## Impact

- `.github/workflows/release.yml` build job.
- Binary release publishing OpenSpec requirements.
- Local workflow syntax validation.
