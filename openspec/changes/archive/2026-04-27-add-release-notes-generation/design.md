## Context

The release workflow already builds platform archives, creates `checksums.txt`, and publishes all assets through `softprops/action-gh-release`. The missing piece is a stable release body that tells users which archive to choose, how to install, and how to verify downloads.

## Goals / Non-Goals

**Goals:**

- Generate a Markdown release body inside the existing tag-driven release workflow.
- Include bilingual install guidance, platform asset names, checksum verification guidance, and an automatically generated changes section.
- Keep archive names, checksum generation, and supported platform targets unchanged.

**Non-Goals:**

- Change CLI behavior or binary contents.
- Add new package manager publishing.
- Add a manually maintained changelog file.

## Decisions

- Generate `dist/release-notes.md` in the release job after `checksums.txt` is created. This keeps the notes next to the assets and lets `softprops/action-gh-release` publish the file via `body_path`.
- Use `gh api repos/{owner}/{repo}/releases/generate-notes` to populate the changes section. This uses GitHub's existing release-note generation instead of maintaining custom git range logic in shell.
- Keep the top section hand-written in the workflow. This guarantees that every release includes the same install, asset, and checksum instructions even when GitHub's generated change summary is sparse.

## Risks / Trade-offs

- GitHub-generated change summaries depend on tag history and merged pull request metadata. Mitigation: the workflow still publishes a deterministic body with install, asset, and verification guidance even when the generated section is short.
- The release job now depends on the `gh` CLI available on `ubuntu-24.04`. Mitigation: `gh` is standard on GitHub-hosted Ubuntu runners, and the workflow remains limited to the release job.
