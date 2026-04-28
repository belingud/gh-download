## Why

The README files already document Homebrew installation, but generated GitHub Release notes still mention only Cargo and prebuilt archives. The repository skill also reports an older tool version and omits Homebrew from its installation guidance.

## What Changes

- Add the Homebrew tap command to generated release notes installation guidance in English and Chinese.
- Update `skills/gh-download/SKILL.md` version metadata to match the crate version.
- Add Homebrew installation guidance to the repository skill.
- Keep CLI behavior, packaging, checksums, and release asset names unchanged.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `binary-release-publishing`: release notes installation guidance must include Homebrew, Cargo, and prebuilt archive options.
- `open-source-entrypoints`: repository skills must keep install guidance aligned with README installation entrypoints and current package version metadata.

## Impact

- `.github/workflows/release.yml` release notes generation step.
- `skills/gh-download/SKILL.md`.
- OpenSpec requirements for release notes and open-source entrypoints.
