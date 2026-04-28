## Context

The README files already list Homebrew as the first installation path, but generated GitHub Release notes still mention only Cargo and prebuilt archives. The repository skill is another user entrypoint for agents and still advertises version `0.4.0` while `Cargo.toml` is at `0.5.0`.

This change keeps the install story consistent across generated release notes, README entrypoints, and the repository skill.

## Goals / Non-Goals

**Goals:**

- Include `brew install belingud/tap/gh-download` in generated release notes.
- Update the `gh-download` skill version metadata to match the crate version.
- Add Homebrew to the skill installation instructions.

**Non-Goals:**

- Change the CLI behavior, packaging matrix, or release asset names.
- Add a new installer or release target.
- Rewrite the skill beyond the stale version and install guidance.

## Decisions

- Keep release notes generation inline in `.github/workflows/release.yml`.
  - Rationale: the existing release notes template is already inline, and this is a small wording update.
  - Alternative considered: move release notes generation to a script. That would add structure without reducing enough complexity for this change.

- Treat the repository skill as an open-source entrypoint.
  - Rationale: it is user-facing guidance for agents, so its version and installation guidance should remain aligned with README and package metadata.
  - Alternative considered: leave the skill outside specs. That would allow the same drift to return.

## Risks / Trade-offs

- Release notes text can drift again when new install paths are added -> encode the expected install methods in the release publishing spec.
- Skill version metadata requires manual updates when the crate version changes -> make that expectation explicit in the open-source entrypoint spec.
