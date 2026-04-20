## 1. CLI Contract

- [x] 1.1 Add an explicit `--json` CLI flag and update localized help text and parsing tests.
- [x] 1.2 Refactor output mode selection so the CLI can switch cleanly between human-readable output and machine-readable JSON output.

## 2. JSON Output

- [x] 2.1 Define and implement the JSON success payload for file and directory downloads, including saved path and aggregate download statistics.
- [x] 2.2 Define and implement the JSON failure payload using the existing classified error information without mixing human-readable stdout output into JSON mode.
- [x] 2.3 Preserve debug diagnostics on stderr when `--json` and `--debug` are enabled together.
- [x] 2.4 Add tests covering JSON success output, JSON failure output, and combined `--json --debug` behavior.

## 3. Docs And Verification

- [x] 3.1 Update `README.md` and `README.zh.md` to document `--json`, its stream behavior, and example payloads.
- [x] 3.2 Run `just fmt`, `just test`, and `just check` to verify the JSON output change.
