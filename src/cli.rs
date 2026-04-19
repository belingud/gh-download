use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

use crate::download::DEFAULT_GH_PROXY;
use crate::error::AppError;
use crate::i18n::{Language, detect_language_from_args_and_env};

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

    #[arg(long = "lang", value_enum, value_name = "LANG")]
    pub language: Option<Language>,

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
    pub language: Language,
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
    let proxy_base = resolve_proxy_base(cli.proxy_base.as_deref(), env::var("GH_PROXY_BASE").ok());
    let token = pick_token(
        cli.token.as_deref(),
        env::var("GITHUB_TOKEN").ok().as_deref(),
        env::var("GH_TOKEN").ok().as_deref(),
    );
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
        language,
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

pub fn resolve_proxy_base(explicit: Option<&str>, env_value: Option<String>) -> String {
    match explicit {
        Some(value) => value.trim().to_string(),
        None => env_value
            .map(|value| value.trim().to_string())
            .unwrap_or_else(|| DEFAULT_GH_PROXY.to_string()),
    }
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
        .mut_arg("language", |arg| arg.help(language_help(language)))
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
            "Proxy prefix used for anonymous fallback requests. Defaults to GH_PROXY_BASE or the built-in proxy; pass an empty string to disable it"
        }
        Language::Zh => {
            "匿名请求回退时使用的代理前缀。默认读取 GH_PROXY_BASE 或使用内置代理；传空字符串可关闭"
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

fn no_color_help(language: Language) -> &'static str {
    match language {
        Language::En => "Disable ANSI colors",
        Language::Zh => "关闭 ANSI 彩色输出",
    }
}

#[cfg(test)]
mod tests {
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
    fn cli_parses_no_color_and_ref() {
        let cli = Cli::try_parse_from([
            "gh-download",
            "owner/repo",
            "README.md",
            "./README.md",
            "--ref",
            "main",
            "--lang",
            "zh",
            "--no-color",
        ])
        .expect("cli should parse");

        assert_eq!(cli.git_ref.as_deref(), Some("main"));
        assert_eq!(cli.language, Some(Language::Zh));
        assert!(cli.no_color);
    }

    #[test]
    fn help_is_localized_for_chinese() {
        let mut command = command_for_language(Language::Zh);
        let rendered = command.render_help().to_string();
        assert!(rendered.contains("用法:"));
        assert!(rendered.contains("显式指定用户可见语言"));
    }
}
