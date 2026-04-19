## Context

`gh-download` is a single-command CLI built on clap with three required positional arguments. Today, `parse_cli_from_env()` forwards the raw process arguments into clap, so invoking the binary with no user-provided arguments produces clap's missing-arguments error instead of the help screen. The requested behavior change is intentionally narrow: only a truly empty invocation should behave like `--help`, while partially specified invocations must keep failing so users do not accidentally mask real mistakes.

## Goals / Non-Goals

**Goals:**
- Make `gh-download` print the existing localized help output when launched with no user-provided arguments.
- Preserve clap's current help formatting, exit semantics, and locale-aware text.
- Keep validation unchanged for invocations that include any user-provided arguments but still miss required positionals.

**Non-Goals:**
- Redesign the CLI interface, argument names, or help template.
- Change behavior for partial invocations such as `gh-download owner/repo` or option-only invocations such as `gh-download --lang zh`.
- Introduce subcommands or a different onboarding flow.

## Decisions

### Detect the empty invocation before clap parsing
Normalize the raw `env::args_os()` vector in `parse_cli_from_env()` and append `--help` only when the vector contains just the executable name.

Rationale:
- This isolates the behavior change to the entry point that knows whether the process was launched with zero user arguments.
- It avoids making required positional arguments optional in the clap definition, which would weaken validation for normal use.

Alternatives considered:
- Print help manually from `main.rs`: rejected because it would duplicate clap output and bypass existing localization and formatting rules.
- Make positionals optional and validate after parsing: rejected because it spreads validation into application code and risks changing partial-invocation behavior.

### Reuse the existing language detection path
Keep the current order in `parse_cli_from_env()`: detect the language first, then parse arguments with a language-specific clap command. The injected `--help` should be parsed by the same localized command instance already used today.

Rationale:
- This preserves the current English/Chinese help rendering without adding a separate help-printing path.
- It keeps `--help` behavior consistent whether the flag was provided by the user or injected for an empty invocation.

Alternatives considered:
- Inject `--help` before language detection: rejected because the empty invocation would no longer flow through the same locale-selection logic.

### Add tests around argument normalization and clap outcomes
Cover the new behavior with tests that verify an empty invocation is transformed into a help request, and that partial invocations still surface missing-argument errors.

Rationale:
- The behavior hinges on a small preprocessing rule, so tests should pin that rule directly.
- Verifying the unchanged failure mode for partial invocations protects against accidentally broadening the new default behavior.

Alternatives considered:
- Rely only on manual `cargo run` verification: rejected because this is a user-visible CLI contract change.

## Risks / Trade-offs

- `[Risk]` Future requests may want option-only invocations to show help as well. -> Mitigation: keep the rule explicit (`args.len() == 1`) so later changes are easy to reason about and extend intentionally.
- `[Risk]` Help behavior still exits through clap's internal path, which can be awkward to test through `parse_cli_from_env()`. -> Mitigation: factor tests around argument normalization and direct clap command execution rather than process-level exit handling.
- `[Trade-off]` The change improves first-run ergonomics only for the fully empty invocation, not for all malformed invocations. -> Mitigation: keep this scoped behavior aligned with the user request and preserve existing error guidance for real input mistakes.
