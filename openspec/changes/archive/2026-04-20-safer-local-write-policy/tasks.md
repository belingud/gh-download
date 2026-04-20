## 1. CLI Contract

- [x] 1.1 Add an explicit `--overwrite` CLI flag and update localized help text and parsing tests.
- [x] 1.2 Update option resolution so overwrite mode is available to both direct file downloads and directory downloads.

## 2. Local Write Behavior

- [x] 2.1 Refactor local file write handling so resolved file targets are skipped by default when they already exist.
- [x] 2.2 Apply the same skip-versus-overwrite policy to direct file downloads and per-file directory download writes without changing target path resolution.
- [x] 2.3 Track skipped existing files separately from skipped unsupported repository entries and surface that distinction in progress and completion output.
- [x] 2.4 Add tests covering direct file conflicts, directory download conflicts, default skip behavior, and explicit `--overwrite` behavior.

## 3. Docs And Verification

- [x] 3.1 Update `README.md` and `README.zh.md` to document the new default skip behavior and the `--overwrite` migration path.
- [x] 3.2 Run `just fmt`, `just test`, and `just check` to verify the safer local write policy change.
