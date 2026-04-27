## Why

Tagged releases currently publish archives and checksums, but the GitHub Release body does not give users a stable explanation of what was published or how to verify the assets.
Adding generated release notes makes each release easier to inspect without changing CLI behavior.

## What Changes

- Generate a release notes Markdown file during the release workflow.
- Publish the generated notes as the GitHub Release body.
- Include the version tag, supported asset list, checksum verification guidance, and short English and Chinese installation guidance.
- Keep generated release assets and checksum publishing unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `binary-release-publishing`: release publishing must include a generated GitHub Release body describing assets, installation, and checksum verification.

## Impact

- `.github/workflows/release.yml`
- `openspec/specs/binary-release-publishing/spec.md`
