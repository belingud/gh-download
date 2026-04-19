## MODIFIED Requirements

### Requirement: CLI accepts GitHub path download inputs
The `gh-download` CLI SHALL accept a repository identifier, a repository-relative remote path, and a local target path as positional arguments when a download invocation is provided. When the CLI is invoked without any user-provided arguments, it SHALL print the localized help text and exit successfully instead of reporting missing required positional arguments. The CLI SHALL also support `--ref`, `--token`, `--proxy-base`, and `--lang` options, and it MUST use `GITHUB_TOKEN` or `GH_TOKEN` as the default token source when `--token` is not provided.

#### Scenario: User provides explicit CLI arguments
- **WHEN** a user runs `gh-download owner/repo src ./downloads --ref main --token abc --proxy-base https://gh-proxy.com/ --lang en`
- **THEN** the CLI accepts the invocation and uses the provided repository, remote path, local target, ref, token, proxy base, and language for the download operation

#### Scenario: User relies on environment token defaults
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md` with `GITHUB_TOKEN` or `GH_TOKEN` set
- **THEN** the CLI uses the environment token automatically without requiring `--token`

#### Scenario: User runs the CLI without arguments
- **WHEN** a user runs `gh-download` without any additional arguments
- **THEN** the CLI prints the help output for the effective language and exits successfully without a missing-arguments error
