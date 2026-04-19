# AGENTS.md

## Project Positioning

`gh-download` is a Rust CLI product for downloading either a file or a directory from a GitHub repository path.

This repository is not a script repo. Treat it as a maintained product repo. Changes that affect user-visible behavior must keep code, docs, tests, and product contract aligned.

## Stable Product Constraints

- The CLI contract and download behavior are part of the product surface, not just implementation details.
- User-visible behavior changes should not be made in code alone.
- The tool must continue to support both file download and recursive directory download.
- Downloads should continue streaming to disk; do not redesign the implementation around full in-memory buffering.
- English and Chinese user-facing behavior should stay semantically aligned.

## Stable Download And Proxy Contract

- Download semantics and proxy behavior are stable product rules and should not be changed casually.
- The tool should determine whether the remote target is a file or directory from GitHub metadata before downloading content.
- File downloads must continue to support both writing to an explicit file path and writing into an existing local directory.
- Directory downloads must preserve relative path structure when saved locally.
- Anonymous raw file downloads may use a URL-prefix proxy such as `gh-proxy`, but that proxy behavior should remain explicit and bounded.
- GitHub metadata API requests must not be routed or retried through `gh-proxy`.
- Authenticated requests must not forward credentials to a public proxy path.
- Ambient system proxy variables are not part of the product contract; do not redesign behavior around them without explicitly changing the contract.

## Source Of Truth

Use the current source code and active specs as the primary contract.

- Runtime behavior lives in `src/`
- User documentation lives in `README.md` and `README.zh.md`
- Active product specs live under `openspec/specs/`

If code and docs/specs diverge, treat that as a problem to resolve rather than assuming one side can be ignored.

## Agent Guidance

- Prefer high-signal, durable changes over edits that only reshuffle wording or structure.
- Avoid putting transient implementation details into persistent project guidance.
- When behavior changes, assume tests need to change too.
- When user-facing behavior changes, review whether docs and active specs also need updates.
- Prefer local, repeatable tests and avoid depending on live network behavior where practical.
- Do not assume file layout is stable; inspect the current tree instead of relying on historical structure notes.

## Verification

Common local verification commands:

- `just fmt`
- `just test`
- `just check`

## What Does Not Belong Here

This file should stay focused on stable rules and collaboration constraints.

- Do not use `AGENTS.md` as a detailed repository map.
- Do not list file-by-file change coupling rules that are likely to break during refactors.
- Do not duplicate workflow detail that belongs in `openspec/`, code comments, or other dedicated docs.
