use std::env;
use std::path::{Path, PathBuf};

use crate::download::DEFAULT_GH_PROXY;
use crate::error::AppError;
use crate::i18n::Language;

use super::types::{Cli, PrefixProxyMode, ResolvedOptions};

const DEBUG_ENV_VAR: &str = "GH_DOWNLOAD_DEBUG";
const PREFIX_MODE_ENV_VAR: &str = "GH_DOWNLOAD_PREFIX_MODE";

pub fn resolve_cli(cli: Cli) -> Result<ResolvedOptions, AppError> {
    if cli.repo.trim().is_empty() {
        return Err(AppError::EmptyRepository);
    }
    if cli.remote_path.trim().is_empty() {
        return Err(AppError::EmptyRemotePath);
    }

    let local_target = resolve_local_target(&cli.local_target)?;
    let prefix_mode = resolve_prefix_mode(
        cli.prefix_mode,
        env::var(PREFIX_MODE_ENV_VAR).ok().as_deref(),
    );
    let proxy_base = resolve_proxy_base(
        cli.proxy_base.as_deref(),
        env::var("GH_PROXY_BASE").ok(),
        prefix_mode,
    );
    let token = pick_token(
        cli.token.as_deref(),
        env::var("GITHUB_TOKEN").ok().as_deref(),
        env::var("GH_TOKEN").ok().as_deref(),
    );
    let debug = resolve_debug(cli.debug, env::var(DEBUG_ENV_VAR).ok().as_deref());
    let language = Language::detect(
        cli.language,
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
        proxy_base,
        prefix_mode,
        concurrency: cli.concurrency,
        language,
        overwrite: cli.overwrite,
        debug,
        no_color: cli.no_color,
    })
}

pub fn pick_token(
    explicit: Option<&str>,
    github_token: Option<&str>,
    gh_token: Option<&str>,
) -> Option<String> {
    selected_token(explicit, github_token, gh_token).map(|(value, _)| value.to_string())
}

pub(crate) fn debug_token_source_label(
    explicit: Option<&str>,
    github_token: Option<&str>,
    gh_token: Option<&str>,
) -> &'static str {
    selected_token(explicit, github_token, gh_token)
        .map(|(_, source)| source)
        .unwrap_or("none")
}

pub fn resolve_proxy_base(
    explicit: Option<&str>,
    env_value: Option<String>,
    prefix_mode: PrefixProxyMode,
) -> String {
    match explicit {
        Some(value) => value.trim().to_string(),
        None => env_value
            .map(|value| value.trim().to_string())
            .unwrap_or_else(|| match prefix_mode {
                PrefixProxyMode::Direct => String::new(),
                PrefixProxyMode::Fallback | PrefixProxyMode::Prefer => DEFAULT_GH_PROXY.to_string(),
            }),
    }
}

pub fn resolve_prefix_mode(
    explicit: Option<PrefixProxyMode>,
    env_value: Option<&str>,
) -> PrefixProxyMode {
    explicit
        .or_else(|| parse_prefix_mode_env(env_value))
        .unwrap_or_default()
}

pub fn resolve_debug(explicit: bool, env_value: Option<&str>) -> bool {
    explicit || parse_bool_env(env_value)
}

pub fn resolve_local_target(path: &Path) -> Result<PathBuf, AppError> {
    let expanded = expand_home(path)?;
    if expanded.is_absolute() {
        Ok(expanded)
    } else {
        env::current_dir()
            .map(|current| current.join(expanded))
            .map_err(|err| AppError::InvalidPath(err.to_string()))
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
    github_token: Option<&'a str>,
    gh_token: Option<&'a str>,
) -> Option<(&'a str, &'static str)> {
    explicit
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| (value, "--token"))
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

fn expand_home(path: &Path) -> Result<PathBuf, AppError> {
    let raw = path.to_string_lossy();
    if raw == "~" {
        return home_dir()
            .ok_or_else(|| AppError::InvalidPath("failed to resolve home directory".to_string()));
    }
    if let Some(suffix) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
        let home = home_dir()
            .ok_or_else(|| AppError::InvalidPath("failed to resolve home directory".to_string()))?;
        return Ok(home.join(suffix));
    }
    Ok(path.to_path_buf())
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn token_priority_prefers_explicit_then_github_then_gh() {
        assert_eq!(
            pick_token(Some("explicit"), Some("github"), Some("gh")),
            Some("explicit".to_string())
        );
        assert_eq!(
            pick_token(None, Some("github"), Some("gh")),
            Some("github".to_string())
        );
        assert_eq!(pick_token(None, None, Some("gh")), Some("gh".to_string()));
    }

    #[test]
    fn debug_token_source_reports_explicit_then_env_precedence() {
        assert_eq!(
            debug_token_source_label(Some("explicit"), Some("github"), Some("gh")),
            "--token"
        );
        assert_eq!(
            debug_token_source_label(None, Some("github"), Some("gh")),
            "GITHUB_TOKEN"
        );
        assert_eq!(debug_token_source_label(None, None, Some("gh")), "GH_TOKEN");
        assert_eq!(debug_token_source_label(None, None, None), "none");
    }

    #[test]
    fn proxy_base_is_disabled_by_default_in_direct_mode() {
        assert_eq!(resolve_proxy_base(None, None, PrefixProxyMode::Direct), "");
    }

    #[test]
    fn proxy_base_uses_env_when_present() {
        assert_eq!(
            resolve_proxy_base(
                None,
                Some(" https://proxy.example/ ".to_string()),
                PrefixProxyMode::Prefer
            ),
            "https://proxy.example/"
        );
    }

    #[test]
    fn proxy_base_explicit_empty_disables_env() {
        assert_eq!(
            resolve_proxy_base(
                Some("  "),
                Some("https://proxy.example/".to_string()),
                PrefixProxyMode::Prefer
            ),
            ""
        );
    }

    #[test]
    fn proxy_base_defaults_to_builtin_proxy_in_fallback_mode() {
        assert_eq!(
            resolve_proxy_base(None, None, PrefixProxyMode::Fallback),
            DEFAULT_GH_PROXY
        );
    }

    #[test]
    fn proxy_base_defaults_to_builtin_proxy_in_prefer_mode() {
        assert_eq!(
            resolve_proxy_base(None, None, PrefixProxyMode::Prefer),
            DEFAULT_GH_PROXY
        );
    }

    #[test]
    fn prefix_mode_defaults_to_direct() {
        assert_eq!(resolve_prefix_mode(None, None), PrefixProxyMode::Direct);
    }

    #[test]
    fn prefix_mode_uses_env_when_present() {
        assert_eq!(
            resolve_prefix_mode(None, Some(" prefer ")),
            PrefixProxyMode::Prefer
        );
    }

    #[test]
    fn prefix_mode_prefers_explicit_value() {
        assert_eq!(
            resolve_prefix_mode(Some(PrefixProxyMode::Fallback), Some("prefer")),
            PrefixProxyMode::Fallback
        );
    }

    #[test]
    fn debug_uses_env_when_flag_is_absent() {
        assert!(resolve_debug(false, Some("true")));
        assert!(!resolve_debug(false, Some("0")));
    }

    #[test]
    fn debug_flag_overrides_env() {
        assert!(resolve_debug(true, Some("0")));
    }

    #[test]
    fn resolve_cli_uses_builtin_proxy_for_prefer_mode_when_unset() {
        let cli = Cli {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: PathBuf::from("./README.md"),
            git_ref: None,
            token: None,
            proxy_base: None,
            prefix_mode: Some(PrefixProxyMode::Prefer),
            concurrency: 8,
            language: Some(Language::En),
            overwrite: true,
            debug: false,
            no_color: true,
        };

        let options = resolve_cli(cli).expect("cli should resolve");
        assert_eq!(options.prefix_mode, PrefixProxyMode::Prefer);
        assert_eq!(options.proxy_base, DEFAULT_GH_PROXY);
        assert!(options.overwrite);
    }

    #[test]
    fn resolve_cli_preserves_explicit_concurrency() {
        let cli = Cli {
            repo: "owner/repo".to_string(),
            remote_path: "src".to_string(),
            local_target: PathBuf::from("./downloads"),
            git_ref: None,
            token: None,
            proxy_base: None,
            prefix_mode: None,
            concurrency: 12,
            language: Some(Language::En),
            overwrite: false,
            debug: false,
            no_color: true,
        };

        let options = resolve_cli(cli).expect("cli should resolve");
        assert_eq!(options.concurrency, 12);
    }
}
