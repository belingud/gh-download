## 1. Release Workflow

- [x] 1.1 Add a release-job step that generates `dist/release-notes.md` after checksum generation.
- [x] 1.2 Include version, asset list, bilingual installation guidance, checksum verification guidance, and generated changes in the release notes.
- [x] 1.3 Configure the GitHub Release publishing step to use the generated notes as the release body.

## 2. Specs And Verification

- [x] 2.1 Update the main `binary-release-publishing` spec to include generated release notes.
- [x] 2.2 Validate OpenSpec artifacts and workflow YAML syntax.
- [x] 2.3 Run repository verification commands for formatting and tests.
