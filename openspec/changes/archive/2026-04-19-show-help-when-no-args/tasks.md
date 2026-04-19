## 1. CLI Entry Behavior

- [x] 1.1 Update `src/cli.rs` so an invocation with no user-provided arguments is normalized to `--help` before clap parsing.
- [x] 1.2 Preserve the existing parsing and validation flow for partial or malformed invocations that still provide one or more user arguments.

## 2. Test Coverage

- [x] 2.1 Add tests that cover the empty-invocation normalization and confirm the localized help path is used.
- [x] 2.2 Add or update tests that confirm partial invocations still fail with missing required arguments instead of defaulting to help.

## 3. Documentation

- [x] 3.1 Update `README.md` usage guidance to mention that running `gh-download` with no arguments shows help.
- [x] 3.2 Update `README.zh.md` to describe the same empty-invocation help behavior in Chinese.
