use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, ValueEnum};

use crate::download::DEFAULT_GH_PROXY;
use crate::error::AppError;
use crate::i18n::{Language, detect_language_from_args_and_env};

const DEBUG_ENV_VAR: &str = "GH_DOWNLOAD_DEBUG";
const PREFIX_MODE_ENV_VAR: &str = "GH_DOWNLOAD_PREFIX_MODE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum PrefixProxyMode {
    #[default]
    Direct,
    Fallback,
    Prefer,
}

impl PrefixProxyMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Fallback => "fallback",
            Self::Prefer => "prefer",
        }
    }
}

#[derive(Debug, Parser, Clone)]
#[command(name = "gh-download", version, long_about = None)]
pub struct Cli {
    #[arg()]
    pub repo: String,

    #[arg()]
    pub remote_path: String,

    #[arg()]
    pub local_target: PathBuf,

    #[arg(long = "ref", value_name = "REF")]
    pub git_ref: Option<String>,

    #[arg(long, value_name = "TOKEN")]
    pub token: Option<String>,

    #[arg(long, value_name = "URL")]
    pub proxy_base: Option<String>,

    #[arg(long = "prefix-mode", value_enum, value_name = "MODE")]
    pub prefix_mode: Option<PrefixProxyMode>,

    #[arg(long = "lang", value_enum, value_name = "LANG")]
    pub language: Option<Language>,

    #[arg(long, action = ArgAction::SetTrue)]
    pub debug: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub no_color: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOptions {
    pub repo: String,
    pub remote_path: String,
    pub local_target: PathBuf,
    pub git_ref: Option<String>,
    pub token: Option<String>,
    pub proxy_base: String,
    pub prefix_mode: PrefixProxyMode,
    pub language: Language,
    pub debug: bool,
    pub no_color: bool,
}

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
        language,
        debug,
        no_color: cli.no_color,
    })
}

pub fn pick_token(
    explicit: Option<&str>,
    github_token: Option<&str>,
    gh_token: Option<&str>,
) -> Option<String> {
    explicit
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            github_token
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
        .or_else(|| {
            gh_token
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
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

pub fn parse_cli_from_env() -> Cli {
    let args: Vec<OsString> = env::args_os().collect();
    let language = detect_language_from_args_and_env(
        &args,
        env::var("LC_ALL").ok().as_deref(),
        env::var("LC_MESSAGES").ok().as_deref(),
        env::var("LANG").ok().as_deref(),
    );
    let args = normalize_args_for_parsing(args);
    parse_cli_from_args(args, language)
}

pub fn parse_cli_from_args<I, T>(args: I, language: Language) -> Cli
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args_vec: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let mut command = command_for_language(language);
    let matches = command
        .try_get_matches_from_mut(args_vec)
        .unwrap_or_else(|error| error.exit());
    Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit())
}

fn normalize_args_for_parsing(mut args: Vec<OsString>) -> Vec<OsString> {
    if args.len() == 1 {
        args.push(OsString::from("--help"));
    }
    args
}

pub fn command() -> clap::Command {
    command_for_language(Language::En)
}

pub fn command_for_language(language: Language) -> clap::Command {
    let mut command = Cli::command()
        .help_template(help_template(language))
        .about(command_about(language))
        .after_help(command_after_help(language));

    command = command
        .mut_arg("repo", |arg| arg.help(repo_help(language)))
        .mut_arg("remote_path", |arg| arg.help(remote_path_help(language)))
        .mut_arg("local_target", |arg| arg.help(local_target_help(language)))
        .mut_arg("git_ref", |arg| arg.help(ref_help(language)))
        .mut_arg("token", |arg| arg.help(token_help(language)))
        .mut_arg("proxy_base", |arg| arg.help(proxy_help(language)))
        .mut_arg("prefix_mode", |arg| arg.help(prefix_mode_help(language)))
        .mut_arg("language", |arg| arg.help(language_help(language)))
        .mut_arg("debug", |arg| arg.help(debug_help(language)))
        .mut_arg("no_color", |arg| arg.help(no_color_help(language)));

    command
}

fn command_about(language: Language) -> &'static str {
    match language {
        Language::En => "Download a file or directory from a GitHub repository path",
        Language::Zh => "下载 GitHub 仓库里的单个文件或整个目录",
    }
}

fn command_after_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Examples:\n  gh-download openai/openai-python README.md ./README.md\n  gh-download owner/repo src ./downloads --ref main\n  gh-download owner/private-repo docs ./docs --token <token>\n  gh-download owner/repo docs ./docs --lang zh"
        }
        Language::Zh => {
            "示例:\n  gh-download openai/openai-python README.md ./README.md\n  gh-download owner/repo src ./downloads --ref main\n  gh-download owner/private-repo docs ./docs --token <token>\n  gh-download owner/repo docs ./docs --lang zh"
        }
    }
}

fn help_template(language: Language) -> &'static str {
    match language {
        Language::En => {
            "{about-with-newline}\nUsage: {usage}\n\nArguments:\n{positionals}\nOptions:\n{options}{after-help}\n"
        }
        Language::Zh => {
            "{about-with-newline}\n用法: {usage}\n\n参数:\n{positionals}\n选项:\n{options}{after-help}\n"
        }
    }
}

fn repo_help(language: Language) -> &'static str {
    match language {
        Language::En => "GitHub repository in OWNER/REPO format, for example openai/openai-python",
        Language::Zh => "GitHub 仓库，格式为 OWNER/REPO，例如 openai/openai-python",
    }
}

fn remote_path_help(language: Language) -> &'static str {
    match language {
        Language::En => "Path inside the repository, for example README.md or src/openai",
        Language::Zh => "仓库内路径，例如 README.md 或 src/openai",
    }
}

fn local_target_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Local destination path. Files may be written to a file path or an existing directory; directory downloads treat it as the parent directory by default"
        }
        Language::Zh => "本地目标路径。文件可写入文件路径或现有目录；目录下载时默认作为父目录",
    }
}

fn ref_help(language: Language) -> &'static str {
    match language {
        Language::En => "Branch, tag, or commit SHA",
        Language::Zh => "分支、tag 或 commit SHA",
    }
}

fn token_help(language: Language) -> &'static str {
    match language {
        Language::En => "GitHub token. Defaults to GITHUB_TOKEN or GH_TOKEN",
        Language::Zh => "GitHub token。默认读取 GITHUB_TOKEN 或 GH_TOKEN",
    }
}

fn proxy_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "URL-prefix proxy base for anonymous raw downloads. Defaults to GH_PROXY_BASE; in fallback/prefer mode it falls back to the built-in gh-proxy when unset"
        }
        Language::Zh => {
            "匿名 raw 下载使用的 URL 前缀代理。默认读取 GH_PROXY_BASE；在 fallback/prefer 模式下未设置时会回退到内置 gh-proxy"
        }
    }
}

fn prefix_mode_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Raw download prefix-proxy mode: direct, fallback, or prefer. Defaults to GH_DOWNLOAD_PREFIX_MODE or direct"
        }
        Language::Zh => {
            "raw 下载的前缀代理模式：direct、fallback 或 prefer。默认读取 GH_DOWNLOAD_PREFIX_MODE，未设置时为 direct"
        }
    }
}

fn language_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Force the user-facing language. Defaults to English unless the locale indicates Chinese"
        }
        Language::Zh => "显式指定用户可见语言。默认英文；当 locale 指向中文时自动切换为中文",
    }
}

fn debug_help(language: Language) -> &'static str {
    match language {
        Language::En => {
            "Print debug diagnostics for request URLs and download strategy. Defaults to GH_DOWNLOAD_DEBUG"
        }
        Language::Zh => "打印请求 URL 和下载策略的调试信息。默认读取 GH_DOWNLOAD_DEBUG",
    }
}

fn no_color_help(language: Language) -> &'static str {
    match language {
        Language::En => "Disable ANSI colors",
        Language::Zh => "关闭 ANSI 彩色输出",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

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
    fn cli_parses_no_color_ref_prefix_mode_and_debug() {
        let cli = Cli::try_parse_from([
            "gh-download",
            "owner/repo",
            "README.md",
            "./README.md",
            "--ref",
            "main",
            "--prefix-mode",
            "prefer",
            "--lang",
            "zh",
            "--debug",
            "--no-color",
        ])
        .expect("cli should parse");

        assert_eq!(cli.git_ref.as_deref(), Some("main"));
        assert_eq!(cli.prefix_mode, Some(PrefixProxyMode::Prefer));
        assert_eq!(cli.language, Some(Language::Zh));
        assert!(cli.debug);
        assert!(cli.no_color);
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
            language: Some(Language::En),
            debug: false,
            no_color: true,
        };

        let options = resolve_cli(cli).expect("cli should resolve");
        assert_eq!(options.prefix_mode, PrefixProxyMode::Prefer);
        assert_eq!(options.proxy_base, DEFAULT_GH_PROXY);
    }

    #[test]
    fn help_is_localized_for_chinese() {
        let mut command = command_for_language(Language::Zh);
        let rendered = command.render_help().to_string();
        assert!(rendered.contains("用法:"));
        assert!(rendered.contains("显式指定用户可见语言"));
    }

    #[test]
    fn empty_invocation_is_normalized_to_help() {
        let args = normalize_args_for_parsing(vec![OsString::from("gh-download")]);

        assert_eq!(
            args,
            vec![OsString::from("gh-download"), OsString::from("--help")]
        );
    }

    #[test]
    fn empty_invocation_uses_localized_help_flow() {
        let args = normalize_args_for_parsing(vec![OsString::from("gh-download")]);
        let mut command = command_for_language(Language::Zh);
        let error = command
            .try_get_matches_from_mut(args)
            .expect_err("empty invocation should display help");

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
        let rendered = error.to_string();
        assert!(rendered.contains("用法:"));
        assert!(rendered.contains("下载 GitHub 仓库里的单个文件或整个目录"));
    }

    #[test]
    fn partial_invocation_still_requires_missing_arguments() {
        let args = normalize_args_for_parsing(vec![
            OsString::from("gh-download"),
            OsString::from("owner/repo"),
        ]);
        let mut command = command_for_language(Language::En);
        let error = command
            .try_get_matches_from_mut(args)
            .expect_err("partial invocation should still fail");

        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
        let rendered = error.to_string();
        assert!(rendered.contains("<REMOTE_PATH>"));
        assert!(rendered.contains("<LOCAL_TARGET>"));
    }
}
