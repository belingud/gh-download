## Why

The project now has a Homebrew tap install command, but the primary README entrypoints still only show Cargo and prebuilt release archives.
Documenting Homebrew gives macOS and Linux users a shorter installation path while keeping existing installation options available.

## What Changes

- Add `brew install belingud/tap/gh-download` to the English and Chinese Quick Start sections.
- Add a Homebrew subsection to the English and Chinese installation sections.
- Keep Cargo, GitHub Releases, and source build instructions available.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `open-source-entrypoints`: README installation entrypoints must include the Homebrew tap command alongside existing install options.

## Impact

- `README.md`
- `README.zh.md`
- `openspec/specs/open-source-entrypoints/spec.md`
