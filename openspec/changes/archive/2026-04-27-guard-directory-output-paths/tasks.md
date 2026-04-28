## 1. Implementation

- [x] 1.1 Add a path helper that validates directory metadata paths before local joins.
- [x] 1.2 Use the validated helper when building directory download jobs.
- [x] 1.3 Keep existing valid nested directory behavior unchanged.

## 2. Tests

- [x] 2.1 Add unit tests for valid directory-relative paths.
- [x] 2.2 Add unit tests that reject parent-directory, absolute, separator-alias, empty, and outside-root paths.
- [x] 2.3 Add a download job construction test that returns `InvalidPath` for unsafe metadata.

## 3. Verification

- [x] 3.1 Run OpenSpec validation for the change and all active specs.
- [x] 3.2 Run Rust formatting, tests, and checks.
