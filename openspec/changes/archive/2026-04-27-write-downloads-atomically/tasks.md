## 1. Implementation

- [x] 1.1 Add a shared transport helper that writes response bytes to a sibling temporary file before committing the final target.
- [x] 1.2 Ensure failed body copies remove the temporary file where possible and do not create or truncate the final target.
- [x] 1.3 Preserve overwrite behavior by replacing existing final files only after the replacement body has copied successfully.

## 2. Tests

- [x] 2.1 Add unit tests for successful atomic commits to new and existing targets.
- [x] 2.2 Add unit tests for failed body copies with no existing target and with an existing target.

## 3. Verification

- [x] 3.1 Run OpenSpec validation for the change and all active specs.
- [x] 3.2 Run Rust formatting, tests, and checks.
