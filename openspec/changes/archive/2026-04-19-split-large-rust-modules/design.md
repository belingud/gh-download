## Context

The current runtime code is concentrated in a few top-level files. `src/download.rs` contains download orchestration, GitHub Contents API modeling, HTTP request construction, raw-file strategy selection, filesystem writes, path utilities, URL redaction, and tests. `src/cli.rs` contains clap definitions, option resolution, environment precedence, path expansion, localized help text, and tests.

The refactor must preserve the existing product contract. The CLI behavior, output text, error guidance, locale handling, proxy behavior, download streaming behavior, path resolution rules, and release flow are all out of scope for behavioral changes.

## Goals / Non-Goals

**Goals:**

- Reduce oversized Rust source files by moving cohesive concerns into smaller modules.
- Keep the crate's current public API surface compatible for existing tests and local consumers.
- Make future behavior changes easier to review by separating orchestration from low-level helpers.
- Keep tests close to the behavior they cover without weakening existing coverage.
- Preserve `cargo fmt`, `cargo test`, and `cargo check` results.

**Non-Goals:**

- No CLI argument, default, help text, output wording, locale, or error-message changes.
- No download behavior changes, including proxy boundaries, credential handling, filesystem layout, or streaming semantics.
- No dependency changes unless implementation uncovers a compelling reason, which is not expected.
- No README, main spec, or release workflow changes unless a behavioral change is accidentally discovered and intentionally accepted.

## Decisions

1. Split by responsibility, not by arbitrary line count.
Rationale: smaller files only help if each module has a clear reason to exist. The target structure should isolate cohesive concerns such as CLI resolution, localized help text, HTTP transport, path utilities, and raw download strategy.
Alternatives considered:
- Move chunks into `download_part1.rs` / `download_part2.rs`: rejected because it lowers line count without improving maintainability.
- Rewrite the runner around new abstractions: rejected because this change should minimize behavioral risk.

2. Keep existing public re-exports stable from `src/lib.rs`.
Rationale: current tests and potential local callers import helpers such as `build_contents_api_url`, `choose_directory_target`, `join_proxy_url`, `normalize_repo_path`, and CLI resolution functions. The internal module layout can change without forcing public call sites to change.
Alternatives considered:
- Make helpers private during refactor: rejected because it mixes API cleanup with structural reorganization.
- Expose every new submodule publicly: rejected because it leaks internal organization as API.

3. Preserve the `Runner` as the orchestration boundary.
Rationale: `Runner::run` is already the main runtime entrypoint. The refactor should move supporting concerns behind it while keeping the control flow recognizable.
Alternatives considered:
- Replace `Runner` with multiple service objects immediately: rejected as too broad for a no-behavior-change refactor.
- Keep all helper functions in one file and only extract tests: rejected because the production file would remain too large.

4. Move tests with their subject modules where practical.
Rationale: focused module tests make it clearer which behavior each file owns. Integration-style tests that exercise the full download runner can remain with the runner module.
Alternatives considered:
- Keep all tests in one large `download` test module: rejected because it preserves much of the current navigation problem.
- Convert all tests to external integration tests: rejected because many tests need direct access to crate-private internals.

5. Treat docs and specs as unchanged unless implementation changes behavior.
Rationale: this change is intentionally internal. Updating user docs for no visible behavior change creates noise and can imply a contract change that does not exist.
Alternatives considered:
- Add a new architecture capability spec: rejected because OpenSpec specs describe product behavior, not internal file layout.

## Risks / Trade-offs

- [Accidental behavior drift while moving code] -> Move code in small slices, keep function bodies intact where possible, and run the existing test suite after each meaningful phase.
- [Public API breakage through changed visibility or paths] -> Preserve current `src/lib.rs` re-exports and add compatibility `pub use` statements from internal modules as needed.
- [Test coverage becomes harder to follow after relocation] -> Keep runner-level tests for end-to-end download behavior and module-level tests for pure helper functions.
- [Circular module dependencies] -> Keep dependency direction simple: CLI modules may depend on download constants, download orchestration may depend on output/error/CLI resolved options, and low-level utility modules should not depend on the runner.
- [Refactor diff becomes too large to review] -> Split implementation tasks by module area and avoid opportunistic rewrites.

## Migration Plan

This is an internal code migration only. Implementation should preserve the binary name, crate exports, command-line interface, and behavior. Rollback is straightforward: revert the structural refactor commit if validation exposes regressions.

## Open Questions

None.
