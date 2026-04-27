## Context

The README files already document Cargo, GitHub Releases, and source builds. The Homebrew tap command now exists and should be presented near the top because it is the shortest install path for many macOS and Linux users.

## Goals / Non-Goals

**Goals:**

- Add Homebrew installation guidance to both README files.
- Keep Cargo and prebuilt release archive guidance visible.
- Keep English and Chinese documentation semantically aligned.

**Non-Goals:**

- Change CLI behavior.
- Change release automation or package publishing.
- Document package managers other than Homebrew.

## Decisions

- Show Homebrew first in Quick Start and Installation because it is now the simplest command for users with Homebrew installed.
- Keep Cargo immediately after Homebrew so Rust users still see the crates.io installation path without scrolling far.
- Use the exact command `brew install belingud/tap/gh-download` in both languages to avoid ambiguity.

## Risks / Trade-offs

- Users without Homebrew still need Cargo or GitHub Releases. Mitigation: keep both existing install methods directly below the Homebrew guidance.
