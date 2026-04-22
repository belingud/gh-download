use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueEnum};
use serde::Deserialize;

use crate::i18n::Language;

pub const DEFAULT_DOWNLOAD_CONCURRENCY: usize = 4;

fn parse_concurrency(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "concurrency must be a positive integer".to_string())?;
    if parsed == 0 {
        return Err("concurrency must be at least 1".to_string());
    }
    Ok(parsed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
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
pub(crate) struct ParsedCli {
    #[arg()]
    pub repo: String,

    #[arg()]
    pub remote_path: String,

    #[arg()]
    pub local_target: PathBuf,

    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[arg(long = "ref", value_name = "REF")]
    pub git_ref: Option<String>,

    #[arg(long, value_name = "TOKEN")]
    pub token: Option<String>,

    #[arg(long, value_name = "URL")]
    pub api_base: Option<String>,

    #[arg(long, value_name = "URL")]
    pub proxy_base: Option<String>,

    #[arg(long = "prefix-mode", value_enum, value_name = "MODE")]
    pub prefix_mode: Option<PrefixProxyMode>,

    #[arg(
        short = 'c',
        long,
        value_name = "N",
        value_parser = parse_concurrency
    )]
    pub concurrency: Option<usize>,

    #[arg(long = "lang", value_enum, value_name = "LANG")]
    pub language: Option<Language>,

    #[arg(long, action = ArgAction::SetTrue)]
    pub overwrite: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub json: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub debug: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub no_color: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliInvocation {
    pub cli: Cli,
    pub config_path: Option<PathBuf>,
    pub explicit_concurrency: Option<usize>,
}

#[derive(Debug, Parser, Clone, PartialEq, Eq)]
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
    pub api_base: Option<String>,

    #[arg(long, value_name = "URL")]
    pub proxy_base: Option<String>,

    #[arg(long = "prefix-mode", value_enum, value_name = "MODE")]
    pub prefix_mode: Option<PrefixProxyMode>,

    #[arg(
        short = 'c',
        long,
        value_name = "N",
        default_value_t = DEFAULT_DOWNLOAD_CONCURRENCY,
        value_parser = parse_concurrency
    )]
    pub concurrency: usize,

    #[arg(long = "lang", value_enum, value_name = "LANG")]
    pub language: Option<Language>,

    #[arg(long, action = ArgAction::SetTrue)]
    pub overwrite: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub json: bool,

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
    pub api_base: String,
    pub proxy_base: String,
    pub prefix_mode: PrefixProxyMode,
    pub concurrency: usize,
    pub language: Language,
    pub overwrite: bool,
    pub json: bool,
    pub debug: bool,
    pub no_color: bool,
}

impl From<ParsedCli> for CliInvocation {
    fn from(value: ParsedCli) -> Self {
        let explicit_concurrency = value.concurrency;
        let cli = Cli {
            repo: value.repo,
            remote_path: value.remote_path,
            local_target: value.local_target,
            git_ref: value.git_ref,
            token: value.token,
            api_base: value.api_base,
            proxy_base: value.proxy_base,
            prefix_mode: value.prefix_mode,
            concurrency: explicit_concurrency.unwrap_or(DEFAULT_DOWNLOAD_CONCURRENCY),
            language: value.language,
            overwrite: value.overwrite,
            json: value.json,
            debug: value.debug,
            no_color: value.no_color,
        };

        Self {
            cli,
            config_path: value.config,
            explicit_concurrency,
        }
    }
}
