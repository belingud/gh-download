## Context

Directory downloads enumerate GitHub contents metadata and then build each local file path from the metadata `path` field. Public GitHub should return paths beneath the requested directory, but the CLI also supports custom `--api-base`, so the local path builder should validate metadata before joining it to the destination directory.

This is a narrow safety change. It should affect only directory job construction and path helper tests.

## Goals / Non-Goals

**Goals:**

- Reject directory file entries that do not live beneath the requested remote directory root.
- Reject computed directory-relative output paths that could escape the chosen local directory.
- Keep normal nested directory downloads unchanged.

**Non-Goals:**

- Add a new CLI flag or compatibility mode.
- Change direct file download target resolution.
- Add filesystem canonicalization that requires local paths to exist before download.

## Decisions

- Validate metadata paths before `directory_target.join(...)`.
  - Rationale: this checks the value at the boundary where remote metadata becomes a local path.
  - Alternative considered: validate after joining and canonicalize the result. That requires files and directories to exist and is a poor fit for downloads that create paths.

- Treat valid directory output paths as repository-style relative paths with only normal path components.
  - Rationale: GitHub repository paths use `/`, and local output must not include absolute roots, parent-directory components, current-directory components, or platform separator aliases.
  - Alternative considered: normalize away `.` and `..` segments. That can hide unsafe metadata rather than making the API response fail clearly.

- Return the existing `InvalidPath` error type.
  - Rationale: this keeps user-facing classification and JSON behavior aligned with existing local path failures.
  - Alternative considered: add a new error variant. The current type is enough for this small product rule.

## Risks / Trade-offs

- Repositories can technically contain unusual names such as backslashes on some platforms -> reject them in directory metadata because they can act as separators on Windows.
- A custom API that returns non-GitHub-style paths will fail earlier -> this is intentional because directory output paths must remain bounded by the selected local directory.
