use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueEnum};

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
    pub concurrency: usize,
    pub language: Language,
    pub overwrite: bool,
    pub debug: bool,
    pub no_color: bool,
}
