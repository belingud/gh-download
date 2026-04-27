# Contributing

Thank you for considering a contribution to `gh-download`.

Chinese issues and pull requests are welcome. English is preferred when the change affects public documentation, release notes, or text shown to users, because the project keeps English and Chinese behavior aligned.

## Before Opening An Issue

- Check `README.md` or `README.zh.md` for the current CLI contract.
- Search existing issues to avoid duplicates.
- Do not paste tokens, private raw URLs, or credentials into public issues.
- For download bugs, include the command, operating system, installation method, `gh-download --version`, authentication mode, proxy mode, expected result, actual result, and redacted debug output when available.

## Pull Requests

- Keep changes small and focused.
- Follow the existing Rust and documentation style.
- If CLI behavior, arguments, output, release behavior, or download rules change, update the matching OpenSpec files under `openspec/`.
- If user-facing behavior changes, update both `README.md` and `README.zh.md` when relevant.
- Do not mix repository settings changes with source changes unless the issue explicitly asks for that.

## Local Verification

Common checks:

```bash
just fmt
just test
just check
```

For documentation-only changes, still make sure Markdown links and examples stay accurate.

## Release-Sensitive Changes

Changes to `.github/workflows/release.yml`, archive contents, target triples, or asset naming affect published binaries. Update `openspec/specs/binary-release-publishing/spec.md` or an active delta spec when those contracts change.
