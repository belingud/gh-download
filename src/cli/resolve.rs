use std::env;
use std::path::{Component, Path, PathBuf};

use crate::download::{DEFAULT_GH_PROXY, DEFAULT_GITHUB_API_BASE};
use crate::error::AppError;
use crate::i18n::Language;

use super::config::{FileConfig, expand_home, load_active_config};
use super::types::{Cli, PrefixProxyMode, ResolvedOptions};

const DEBUG_ENV_VAR: &str = "GH_DOWNLOAD_DEBUG";
const PREFIX_MODE_ENV_VAR: &str = "GH_DOWNLOAD_PREFIX_MODE";

pub fn resolve_cli(cli: Cli) -> Result<ResolvedOptions, AppError> {
    let explicit_concurrency = cli.concurrency;
    resolve_cli_with_sources(cli, None, Some(explicit_concurrency))
}

pub(crate) fn resolve_cli_with_config(
    cli: Cli,
    config: Option<FileConfig>,
    explicit_concurrency: Option<usize>,
) -> Result<ResolvedOptions, AppError> {
    if cli.repo.trim().is_empty() {
        return Err(AppError::EmptyRepository);
    }
    if cli.remote_path.trim().is_empty() {
        return Err(AppError::EmptyRemotePath);
    }

    let local_target = resolve_local_target(&cli.local_target)?;
    let prefix_mode = resolve_prefix_mode(
        cli.prefix_mode,
        config.as_ref().and_then(|value| value.prefix_mode),
        env::var(PREFIX_MODE_ENV_VAR).ok().as_deref(),
    );
    let proxy_base = resolve_proxy_base(
        cli.proxy_base.as_deref(),
        config
            .as_ref()
            .and_then(|value| value.proxy_base.as_deref()),
        env::var("GH_PROXY_BASE").ok(),
        prefix_mode,
    );
    let api_base = resolve_api_base(
        cli.api_base.as_deref(),
        config.as_ref().and_then(|value| value.api_base.as_deref()),
    );
    let token = pick_token(
        cli.token.as_deref(),
        config.as_ref().and_then(|value| value.token.as_deref()),
        env::var("GITHUB_TOKEN").ok().as_deref(),
        env::var("GH_TOKEN").ok().as_deref(),
    );
    let debug = resolve_debug(cli.debug, env::var(DEBUG_ENV_VAR).ok().as_deref());
    let language = resolve_language(
        cli.language,
        config.as_ref().and_then(|value| value.lang),
        env::var("LC_ALL").ok().as_deref(),
        env::var("LC_MESSAGES").ok().as_deref(),
        env::var("LANG").ok().as_deref(),
    );

    Ok(ResolvedOptions {
        repo: cli.repo.trim().to_string(),
        remote_path: cli.remote_path.trim().to_string(),
        local_target,
        git_ref: cli.git_ref.map(|value| value.trim().to_string()),
        token,
        api_base,
        proxy_base,
        prefix_mode,
        concurrency: resolve_concurrency(
            explicit_concurrency,
            config.and_then(|value| value.concurrency),
        ),
        language,
        overwrite: cli.overwrite,
        json: cli.json,
        debug,
        no_color: cli.no_color,
    })
}

pub(crate) fn resolve_cli_with_sources(
    cli: Cli,
    config_path: Option<&Path>,
    explicit_concurrency: Option<usize>,
) -> Result<ResolvedOptions, AppError> {
    let config = load_active_config(config_path)?;
    resolve_cli_with_config(cli, config, explicit_concurrency)
}

pub fn pick_token(
    explicit: Option<&str>,
    config_value: Option<&str>,
    github_token: Option<&str>,
    gh_token: Option<&str>,
) -> Option<String> {
    selected_token(explicit, config_value, github_token, gh_token)
        .map(|(value, _)| value.to_string())
}

pub(crate) fn debug_token_source_label(
    explicit: Option<&str>,
    config_value: Option<&str>,
    github_token: Option<&str>,
    gh_token: Option<&str>,
) -> &'static str {
    selected_token(explicit, config_value, github_token, gh_token)
        .map(|(_, source)| source)
        .unwrap_or("none")
}

pub(crate) fn token_present(
    explicit: Option<&str>,
    config_value: Option<&str>,
    github_token: Option<&str>,
    gh_token: Option<&str>,
) -> bool {
    selected_token(explicit, config_value, github_token, gh_token).is_some()
}

pub fn resolve_proxy_base(
    explicit: Option<&str>,
    config_value: Option<&str>,
    env_value: Option<String>,
    prefix_mode: PrefixProxyMode,
) -> String {
    match explicit {
        Some(value) => value.trim().to_string(),
        None => match config_value {
            Some(value) => value.trim().to_string(),
            None => env_value
                .map(|value| value.trim().to_string())
                .unwrap_or_else(|| match prefix_mode {
                    PrefixProxyMode::Direct => String::new(),
                    PrefixProxyMode::Fallback | PrefixProxyMode::Prefer => {
                        DEFAULT_GH_PROXY.to_string()
                    }
                }),
        },
    }
}

pub fn resolve_api_base(explicit: Option<&str>, config_value: Option<&str>) -> String {
    explicit
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            config_value
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or(DEFAULT_GITHUB_API_BASE)
        .to_string()
}

pub fn resolve_prefix_mode(
    explicit: Option<PrefixProxyMode>,
    config_value: Option<PrefixProxyMode>,
    env_value: Option<&str>,
) -> PrefixProxyMode {
    explicit
        .or(config_value)
        .or_else(|| parse_prefix_mode_env(env_value))
        .unwrap_or_default()
}

pub fn resolve_concurrency(explicit: Option<usize>, config_value: Option<usize>) -> usize {
    explicit
        .or(config_value)
        .unwrap_or(super::types::DEFAULT_DOWNLOAD_CONCURRENCY)
}

pub fn resolve_debug(explicit: bool, env_value: Option<&str>) -> bool {
    explicit || parse_bool_env(env_value)
}

pub fn resolve_language(
    explicit: Option<Language>,
    config_value: Option<Language>,
    lc_all: Option<&str>,
    lc_messages: Option<&str>,
    lang: Option<&str>,
) -> Language {
    Language::detect(explicit.or(config_value), lc_all, lc_messages, lang)
}

pub fn resolve_local_target(path: &Path) -> Result<PathBuf, AppError> {
    let expanded = expand_home(path)?;
    if expanded.is_absolute() {
        Ok(normalize_path_lexically(&expanded))
    } else {
        env::current_dir()
            .map(|current| current.join(expanded))
            .map(|joined| normalize_path_lexically(&joined))
            .map_err(|err| AppError::InvalidPath(err.to_string()))
    }
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

fn parse_prefix_mode_env(value: Option<&str>) -> Option<PrefixProxyMode> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) if value.eq_ignore_ascii_case("direct") => Some(PrefixProxyMode::Direct),
        Some(value) if value.eq_ignore_ascii_case("fallback") => Some(PrefixProxyMode::Fallback),
        Some(value) if value.eq_ignore_ascii_case("prefer") => Some(PrefixProxyMode::Prefer),
        _ => None,
    }
}

fn parse_bool_env(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim),
        Some("1")
            | Some("true")
            | Some("TRUE")
            | Some("True")
            | Some("yes")
            | Some("YES")
            | Some("Yes")
            | Some("on")
            | Some("ON")
            | Some("On")
    )
}

fn selected_token<'a>(
    explicit: Option<&'a str>,
    config_value: Option<&'a str>,
    github_token: Option<&'a str>,
    gh_token: Option<&'a str>,
) -> Option<(&'a str, &'static str)> {
    explicit
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| (value, "--token"))
        .or_else(|| {
            config_value
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| (value, "config"))
        })
        .or_else(|| {
            github_token
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| (value, "GITHUB_TOKEN"))
        })
        .or_else(|| {
            gh_token
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| (value, "GH_TOKEN"))
        })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn token_priority_prefers_explicit_then_github_then_gh() {
        assert_eq!(
            pick_token(Some("explicit"), Some("config"), Some("github"), Some("gh")),
            Some("explicit".to_string())
        );
        assert_eq!(
            pick_token(None, Some("config"), Some("github"), Some("gh")),
            Some("config".to_string())
        );
        assert_eq!(
            pick_token(None, None, Some("github"), Some("gh")),
            Some("github".to_string())
        );
        assert_eq!(
            pick_token(None, None, None, Some("gh")),
            Some("gh".to_string())
        );
    }

    #[test]
    fn debug_token_source_reports_explicit_then_env_precedence() {
        assert_eq!(
            debug_token_source_label(Some("explicit"), Some("config"), Some("github"), Some("gh")),
            "--token"
        );
        assert_eq!(
            debug_token_source_label(None, Some("config"), Some("github"), Some("gh")),
            "config"
        );
        assert_eq!(
            debug_token_source_label(None, None, Some("github"), Some("gh")),
            "GITHUB_TOKEN"
        );
        assert_eq!(
            debug_token_source_label(None, None, None, Some("gh")),
            "GH_TOKEN"
        );
        assert_eq!(debug_token_source_label(None, None, None, None), "none");
    }

    #[test]
    fn proxy_base_is_disabled_by_default_in_direct_mode() {
        assert_eq!(
            resolve_proxy_base(None, None, None, PrefixProxyMode::Direct),
            ""
        );
    }

    #[test]
    fn proxy_base_uses_env_when_present() {
        assert_eq!(
            resolve_proxy_base(
                None,
                None,
                Some(" https://proxy.example/ ".to_string()),
                PrefixProxyMode::Prefer
            ),
            "https://proxy.example/"
        );
    }

    #[test]
    fn proxy_base_uses_config_before_env() {
        assert_eq!(
            resolve_proxy_base(
                None,
                Some(" https://config-proxy.example/ "),
                Some("https://env-proxy.example/".to_string()),
                PrefixProxyMode::Prefer
            ),
            "https://config-proxy.example/"
        );
    }

    #[test]
    fn proxy_base_explicit_empty_disables_env() {
        assert_eq!(
            resolve_proxy_base(
                Some("  "),
                None,
                Some("https://proxy.example/".to_string()),
                PrefixProxyMode::Prefer
            ),
            ""
        );
    }

    #[test]
    fn proxy_base_defaults_to_builtin_proxy_in_fallback_mode() {
        assert_eq!(
            resolve_proxy_base(None, None, None, PrefixProxyMode::Fallback),
            DEFAULT_GH_PROXY
        );
    }

    #[test]
    fn proxy_base_defaults_to_builtin_proxy_in_prefer_mode() {
        assert_eq!(
            resolve_proxy_base(None, None, None, PrefixProxyMode::Prefer),
            DEFAULT_GH_PROXY
        );
    }

    #[test]
    fn prefix_mode_defaults_to_direct() {
        assert_eq!(
            resolve_prefix_mode(None, None, None),
            PrefixProxyMode::Direct
        );
    }

    #[test]
    fn api_base_defaults_to_public_github() {
        assert_eq!(resolve_api_base(None, None), DEFAULT_GITHUB_API_BASE);
    }

    #[test]
    fn api_base_uses_trimmed_explicit_value() {
        assert_eq!(
            resolve_api_base(Some(" https://ghe.example.com/api/v3/ "), None),
            "https://ghe.example.com/api/v3/"
        );
    }

    #[test]
    fn api_base_uses_config_before_default() {
        assert_eq!(
            resolve_api_base(None, Some(" https://ghe.example.com/api/v3 ")),
            "https://ghe.example.com/api/v3"
        );
    }

    #[test]
    fn prefix_mode_uses_env_when_present() {
        assert_eq!(
            resolve_prefix_mode(None, None, Some(" prefer ")),
            PrefixProxyMode::Prefer
        );
    }

    #[test]
    fn prefix_mode_prefers_explicit_value() {
        assert_eq!(
            resolve_prefix_mode(Some(PrefixProxyMode::Fallback), None, Some("prefer")),
            PrefixProxyMode::Fallback
        );
    }

    #[test]
    fn prefix_mode_uses_config_before_env() {
        assert_eq!(
            resolve_prefix_mode(None, Some(PrefixProxyMode::Prefer), Some("fallback")),
            PrefixProxyMode::Prefer
        );
    }

    #[test]
    fn concurrency_uses_config_before_default() {
        assert_eq!(resolve_concurrency(None, Some(6)), 6);
        assert_eq!(resolve_concurrency(None, None), 4);
    }

    #[test]
    fn language_uses_config_before_locale() {
        assert_eq!(
            resolve_language(None, Some(Language::En), Some("zh_CN.UTF-8"), None, None),
            Language::En
        );
    }

    #[test]
    fn debug_uses_env_when_flag_is_absent() {
        assert!(resolve_debug(false, Some("true")));
        assert!(!resolve_debug(false, Some("0")));
    }

    #[test]
    fn resolve_cli_uses_config_values_before_env() {
        let cli = Cli {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: PathBuf::from("./README.md"),
            git_ref: None,
            token: None,
            api_base: None,
            proxy_base: None,
            prefix_mode: None,
            concurrency: 4,
            language: None,
            overwrite: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let config = FileConfig {
            token: Some("config-token".to_string()),
            api_base: Some("https://ghe.example.com/api/v3".to_string()),
            proxy_base: Some("https://config-proxy.example/".to_string()),
            prefix_mode: Some(PrefixProxyMode::Prefer),
            concurrency: Some(6),
            lang: Some(Language::En),
        };

        let options = resolve_cli_with_config(cli, Some(config), None).expect("cli should resolve");

        assert_eq!(options.token, Some("config-token".to_string()));
        assert_eq!(options.api_base, "https://ghe.example.com/api/v3");
        assert_eq!(options.proxy_base, "https://config-proxy.example/");
        assert_eq!(options.prefix_mode, PrefixProxyMode::Prefer);
        assert_eq!(options.concurrency, 6);
        assert_eq!(options.language, Language::En);
    }

    #[test]
    fn debug_flag_overrides_env() {
        assert!(resolve_debug(true, Some("0")));
    }

    #[test]
    fn resolve_cli_preserves_explicit_api_base_and_flags() {
        let cli = Cli {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: PathBuf::from("./README.md"),
            git_ref: None,
            token: None,
            api_base: Some(" https://ghe.example.com/api/v3 ".to_string()),
            proxy_base: None,
            prefix_mode: Some(PrefixProxyMode::Prefer),
            concurrency: 8,
            language: Some(Language::En),
            overwrite: true,
            json: true,
            debug: false,
            no_color: true,
        };

        let options = resolve_cli(cli).expect("cli should resolve");
        assert_eq!(options.prefix_mode, PrefixProxyMode::Prefer);
        assert_eq!(options.api_base, "https://ghe.example.com/api/v3");
        assert!(options.overwrite);
        assert!(options.json);
    }

    #[test]
    fn resolve_cli_preserves_explicit_concurrency() {
        let cli = Cli {
            repo: "owner/repo".to_string(),
            remote_path: "src".to_string(),
            local_target: PathBuf::from("./downloads"),
            git_ref: None,
            token: None,
            api_base: None,
            proxy_base: None,
            prefix_mode: None,
            concurrency: 12,
            language: Some(Language::En),
            overwrite: false,
            json: false,
            debug: false,
            no_color: true,
        };

        let options = resolve_cli(cli).expect("cli should resolve");
        assert_eq!(options.concurrency, 12);
    }

    #[test]
    fn local_target_is_lexically_normalized_for_relative_paths() {
        let current_dir = env::current_dir().expect("current dir");

        let resolved = resolve_local_target(Path::new("./var/./nested/../file.txt"))
            .expect("local target should resolve");

        assert_eq!(resolved, current_dir.join("var/file.txt"));
    }

    #[test]
    fn local_target_is_lexically_normalized_for_absolute_paths() {
        assert_eq!(
            resolve_local_target(Path::new("/tmp/./gh-download/../target")).expect("absolute"),
            PathBuf::from("/tmp/target")
        );
    }
}
