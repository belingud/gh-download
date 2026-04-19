## 1. Baseline And Boundaries

- [x] 1.1 Run the existing test suite before refactoring to establish a behavior baseline.
- [x] 1.2 Inventory current crate-root public re-exports from `src/lib.rs` and keep a compatibility checklist for the refactor.
- [x] 1.3 Identify which tests are pure helper tests and which are runner-level behavior tests before moving code.

## 2. Download Module Reorganization

- [x] 2.1 Create a focused `download` module structure while preserving the existing crate-root public exports.
- [x] 2.2 Move GitHub Contents API data models and response parsing helpers into a dedicated download submodule.
- [x] 2.3 Move HTTP client construction, request sending, response-detail extraction, and streaming file writes into a transport-focused submodule without changing timeout, redirect, no-proxy, header, or streaming behavior.
- [x] 2.4 Move path and URL helpers such as repository path normalization, API URL construction, proxy URL joining, URL redaction, relative path calculation, and target selection into focused utility submodules.
- [x] 2.5 Move raw-download strategy selection and debug strategy labels into a focused submodule while preserving direct, fallback, and prefer behavior.
- [x] 2.6 Keep `Runner` as the main orchestration entrypoint and update internal imports to use the new submodules.
- [x] 2.7 Relocate download tests next to the modules they verify, keeping full runner tests for file download, recursive directory download, unsupported-entry skipping, proxy behavior, credential boundaries, debug behavior, and streaming fallback paths.

## 3. CLI Module Reorganization

- [x] 3.1 Create a focused `cli` module structure while preserving existing parsed options and crate-root public exports.
- [x] 3.2 Move clap argument definitions and command construction into a parser/command-focused submodule.
- [x] 3.3 Move resolved option construction, token precedence, proxy-base resolution, prefix-mode resolution, debug resolution, and local target expansion into focused resolver/path submodules.
- [x] 3.4 Move localized CLI help text helpers into a dedicated submodule without changing English or Chinese wording.
- [x] 3.5 Relocate CLI tests next to the modules they verify, preserving coverage for empty invocation help behavior, partial missing arguments, token precedence, env precedence, path expansion, localized help, prefix mode, debug mode, and no-color behavior.

## 4. Compatibility And Cleanup

- [x] 4.1 Update `src/lib.rs` re-exports so existing imports continue to compile after internal module paths change.
- [x] 4.2 Remove obsolete imports and dead code introduced by the file moves.
- [x] 4.3 Confirm that no README, main OpenSpec spec, release workflow, or user-facing output changes were introduced.

## 5. Verification

- [x] 5.1 Run `just fmt`.
- [x] 5.2 Run `just test`.
- [x] 5.3 Run `just check`.
- [x] 5.4 Review the final diff for unintended behavior, documentation, spec, dependency, or release-flow changes.
