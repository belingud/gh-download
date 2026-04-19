## Why

Running `gh-download` without any arguments currently exits with a clap error that says the required positional arguments are missing. For a small single-command CLI, showing the localized help screen by default is a friendlier first-run experience and makes the tool easier to discover from a terminal.

## What Changes

- Treat an invocation with no user-provided arguments as if the user had requested `--help`.
- Keep the existing validation behavior for partially specified invocations, such as providing only one or two positional arguments.
- Update CLI-facing tests and usage documentation to reflect the new empty-invocation behavior.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `github-path-download`: Change the top-level CLI behavior so an empty invocation prints the localized help output instead of a missing-arguments error.

## Impact

- Affects CLI argument parsing in `src/cli.rs`.
- Adds or updates tests that cover zero-argument invocation and exit semantics.
- Requires documentation updates for examples that describe how users discover command usage.
