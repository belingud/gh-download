## ADDED Requirements

### Requirement: CLI discovers configuration from an explicit path or the default config path
The CLI SHALL support an optional `--config <path>` argument that selects the configuration file for the current invocation. When `--config` is provided, the CLI MUST read only that file and it MUST NOT also read the default config path. When `--config` is not provided, the CLI SHALL attempt to read `~/.config/gh-download/config.toml` only if that file exists. The CLI MUST NOT create the default config file automatically. If an explicit config file cannot be read or parsed, the CLI MUST fail before starting the download or printing localized help for that invocation. If the default config file does not exist, the CLI SHALL continue without error.

#### Scenario: Explicit config file replaces the default config path
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --config /tmp/custom.toml`
- **THEN** the CLI reads `/tmp/custom.toml` as the only configuration file for that invocation

#### Scenario: Missing default config file is ignored
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md` and `~/.config/gh-download/config.toml` does not exist
- **THEN** the CLI continues without a configuration-file error

#### Scenario: Missing explicit config file fails the invocation
- **WHEN** a user runs `gh-download owner/repo README.md ./README.md --config /tmp/missing.toml`
- **THEN** the CLI reports a user-visible configuration error instead of continuing with environment variables or built-in defaults

### Requirement: Configuration file format is flat TOML with a bounded key set
The configuration file SHALL use flat top-level TOML keys. It MUST support `token`, `api_base`, `proxy_base`, `prefix_mode`, `concurrency`, and `lang`. It MUST NOT accept positional download inputs such as repository, remote path, or local target. Unknown keys, invalid enum values, and values with invalid types MUST produce a user-visible configuration error instead of being silently ignored.

#### Scenario: Supported TOML keys are accepted
- **WHEN** a configuration file contains valid top-level keys such as `api_base = "https://api.github.com"`, `prefix_mode = "direct"`, `concurrency = 4`, `lang = "zh"`, and `token = "xxxx"`
- **THEN** the CLI accepts those values as configuration defaults for the matching options

#### Scenario: Positional download fields are rejected in config
- **WHEN** a configuration file contains an unsupported key such as `repo = "owner/repo"`
- **THEN** the CLI reports a user-visible configuration error instead of treating that key as a valid input source

#### Scenario: Invalid config value is rejected
- **WHEN** a configuration file contains an invalid value such as `prefix_mode = "proxy-first"` or `concurrency = 0`
- **THEN** the CLI reports a user-visible configuration error instead of starting the download

### Requirement: Supported options merge with CLI arguments, environment sources, and defaults in a fixed order
For config-supported options, the CLI SHALL resolve values using this precedence: explicit CLI argument, then configuration file, then existing environment source, then built-in default. For `token`, the environment source MUST remain `GITHUB_TOKEN` followed by `GH_TOKEN`. For `lang`, the environment source MUST remain locale detection using `LC_ALL`, then `LC_MESSAGES`, then `LANG`. When configuration-file values are present, they MUST override those environment sources unless an explicit CLI argument is also present.

#### Scenario: Config file token overrides token environment variables
- **WHEN** a configuration file contains `token = "config-token"`, `GITHUB_TOKEN` is set to a different value, and the user does not provide `--token`
- **THEN** the CLI uses the configuration-file token for the invocation

#### Scenario: CLI argument overrides config file value
- **WHEN** a configuration file contains `prefix_mode = "fallback"` and the user runs `gh-download owner/repo README.md ./README.md --prefix-mode direct`
- **THEN** the CLI uses `direct` as the effective prefix proxy mode

#### Scenario: Config file language overrides locale
- **WHEN** a configuration file contains `lang = "en"`, the locale indicates Chinese, and the user does not provide `--lang`
- **THEN** the CLI prints help text, status messages, and error guidance in English
