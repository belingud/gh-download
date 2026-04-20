## Context

`gh-download` currently emits a human-oriented stream of startup lines, progress lines, warnings, completion summaries, and user-facing errors. That works well in a terminal, but it is brittle for scripts and AI tooling because the output must be parsed heuristically and can vary with language, color, and progress events.

This change is cross-cutting because it touches CLI parsing, output rendering, error reporting, documentation, and the interaction with the existing debug mode. The repository already treats output wording and CLI behavior as part of the product contract, so JSON mode needs a clear boundary instead of being layered loosely on top of the current text flow.

## Goals / Non-Goals

**Goals:**
- Add an explicit `--json` mode that produces machine-readable success and failure output.
- Keep the current human-readable terminal experience as the default mode.
- Define a stable JSON shape that includes saved path and download statistics useful to automation.
- Keep debug diagnostics compatible with JSON mode without corrupting the JSON payload.

**Non-Goals:**
- Replace the current human-readable output as the default mode.
- Introduce streaming JSON events for every progress step in this change.
- Add schema version negotiation, NDJSON, or multiple machine-readable formats in the first iteration.
- Change download semantics, proxy behavior, or locale selection outside the output contract.

## Decisions

### 1. Add a single final JSON result object instead of streaming progress JSON

When `--json` is enabled, the CLI will emit one structured JSON document describing the final outcome. Human-readable startup, progress, warning, completion, and user-facing error text will be suppressed from stdout in this mode.

Why:
- A single result object is simpler for scripts and AI tools to consume reliably.
- It avoids mixing progress chatter with the final result, which would otherwise require event framing or NDJSON.
- It keeps the initial contract small and testable.

Alternatives considered:
- Stream progress as JSON events: rejected for the first version because it expands the contract substantially and forces event framing decisions early.
- Emit both text and JSON on stdout: rejected because it defeats machine-readability.

### 2. Keep debug diagnostics on stderr even in JSON mode

If `--debug` is enabled together with `--json`, the structured result will still be written to stdout, while debug diagnostics remain on stderr.

Why:
- This preserves the existing purpose of debug output and keeps the JSON payload clean.
- It allows advanced users to collect structured results and diagnostics simultaneously without parsing mixed streams.

Alternatives considered:
- Disable debug entirely in JSON mode: rejected because it removes a useful troubleshooting tool.
- Fold debug diagnostics into the JSON payload: rejected because it couples stable result schema to verbose troubleshooting details.

### 3. Reuse existing runtime statistics and error classification, but serialize them explicitly

The JSON success object should include the resolved saved path and the same aggregate counts the CLI already tracks, including downloaded files, skipped existing files, and skipped unsupported entries. The JSON failure object should include a machine-readable success flag plus the same user-facing title, reason, and suggestions already derived from error classification.

Why:
- It keeps human-readable and machine-readable modes semantically aligned.
- It minimizes duplicate decision logic in the implementation.

Alternatives considered:
- Expose raw internal errors in JSON mode: rejected because the existing classified guidance is part of the product value and easier for automation to surface upstream.

### 4. JSON mode remains language-neutral in field names but may carry localized message text

JSON keys and structural fields will use stable English identifiers, while message-bearing fields such as title, reason, or suggestions may still reflect the effective language. This preserves locale behavior without making schema keys locale-dependent.

Why:
- Tools need stable key names regardless of locale.
- Users still benefit from localized human-readable message content when they inspect JSON directly.

Alternatives considered:
- Force English messages in JSON mode: rejected because it creates a separate locale rule just for one output mode.

## Risks / Trade-offs

- [Users may expect live progress updates in JSON mode] -> Document clearly that the first version emits only a final structured result.
- [Localized message text inside JSON may surprise some integrations] -> Keep schema keys stable and document that message strings follow the effective language.
- [Future schema changes could break automation] -> Keep the first JSON payload small and additive-friendly.
- [Debug plus JSON could still confuse users who merge stdout and stderr] -> Document the stream separation explicitly.

## Migration Plan

1. Add specs for `--json` success/failure output and update existing output-related capabilities.
2. Extend CLI parsing/help with a `--json` flag.
3. Refactor output handling so human-readable rendering can be suppressed while JSON results are serialized at the end of execution.
4. Keep debug diagnostics on stderr and verify they do not contaminate stdout JSON.
5. Add tests for JSON success output, JSON failure output, and combined `--json --debug` behavior.
6. Update English and Chinese docs with examples and stream expectations.

Rollback strategy:
- If the JSON contract proves too ambiguous during implementation, remove the flag before release and revisit with a narrower or versioned schema.

## Open Questions

- None currently.
