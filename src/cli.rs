mod config;
mod help;
mod resolve;
mod types;

use std::env;
use std::ffi::OsString;

use clap::FromArgMatches;

use crate::i18n::Language;

pub use self::help::{command, command_for_language};
pub(crate) use self::resolve::debug_token_source_label;
pub(crate) use self::resolve::token_present;
pub use self::resolve::{
    pick_token, resolve_cli, resolve_debug, resolve_local_target, resolve_prefix_mode,
    resolve_proxy_base,
};
pub(crate) use self::resolve::{resolve_cli_with_config, resolve_language};
pub use self::types::{Cli, CliInvocation, PrefixProxyMode, ResolvedOptions};

pub(crate) use self::config::{detect_language_from_args_env_and_config, load_active_config};

pub fn parse_cli_from_env() -> Cli {
    parse_cli_invocation_from_env().cli
}

pub fn parse_cli_invocation_from_env() -> CliInvocation {
    let args: Vec<OsString> = env::args_os().collect();
    let language = detect_language_from_args_env_and_config(
        &args,
        env::var("LC_ALL").ok().as_deref(),
        env::var("LC_MESSAGES").ok().as_deref(),
        env::var("LANG").ok().as_deref(),
    );
    let args = normalize_args_for_parsing(args);
    parse_cli_invocation_from_args(args, language)
}

pub fn parse_cli_from_args<I, T>(args: I, language: Language) -> Cli
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    parse_cli_invocation_from_args(args, language).cli
}

pub fn parse_cli_invocation_from_args<I, T>(args: I, language: Language) -> CliInvocation
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args_vec: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let mut command = command_for_language(language);
    let matches = command
        .try_get_matches_from_mut(args_vec)
        .unwrap_or_else(|error| error.exit());
    self::types::ParsedCli::from_arg_matches(&matches)
        .unwrap_or_else(|error| error.exit())
        .into()
}

fn normalize_args_for_parsing(mut args: Vec<OsString>) -> Vec<OsString> {
    if args.len() == 1 {
        args.push(OsString::from("--help"));
    }
    args
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use clap::Parser;

    use super::*;

    #[test]
    fn cli_parses_concurrency_json_overwrite_no_color_ref_prefix_mode_and_debug() {
        let invocation = parse_cli_invocation_from_args(
            [
                "gh-download",
                "owner/repo",
                "src",
                "./downloads",
                "--config",
                "./gh-download.toml",
                "--ref",
                "main",
                "--prefix-mode",
                "prefer",
                "--api-base",
                "https://ghe.example.com/api/v3",
                "-c",
                "8",
                "--lang",
                "zh",
                "--overwrite",
                "--json",
                "--debug",
                "--no-color",
            ],
            Language::En,
        );
        let cli = invocation.cli;

        assert_eq!(
            invocation.config_path.as_deref(),
            Some(std::path::Path::new("./gh-download.toml"))
        );
        assert_eq!(cli.git_ref.as_deref(), Some("main"));
        assert_eq!(cli.prefix_mode, Some(PrefixProxyMode::Prefer));
        assert_eq!(
            cli.api_base.as_deref(),
            Some("https://ghe.example.com/api/v3")
        );
        assert_eq!(cli.concurrency, 8);
        assert_eq!(invocation.explicit_concurrency, Some(8));
        assert_eq!(cli.language, Some(Language::Zh));
        assert!(cli.overwrite);
        assert!(cli.json);
        assert!(cli.debug);
        assert!(cli.no_color);
    }

    #[test]
    fn cli_rejects_zero_concurrency() {
        let error = Cli::try_parse_from([
            "gh-download",
            "owner/repo",
            "src",
            "./downloads",
            "--concurrency",
            "0",
        ])
        .expect_err("zero concurrency should be rejected");

        assert!(error.to_string().contains("at least 1"));
    }

    #[test]
    fn cli_accepts_long_concurrency_flag() {
        let cli = parse_cli_from_args(
            [
                "gh-download",
                "owner/repo",
                "src",
                "./downloads",
                "--concurrency",
                "6",
            ],
            Language::En,
        );

        assert_eq!(cli.concurrency, 6);
    }

    #[test]
    fn cli_defaults_concurrency_to_four() {
        let invocation = parse_cli_invocation_from_args(
            ["gh-download", "owner/repo", "src", "./downloads"],
            Language::En,
        );
        let cli = invocation.cli;

        assert_eq!(cli.concurrency, 4);
        assert_eq!(invocation.explicit_concurrency, None);
        assert!(!cli.overwrite);
        assert!(!cli.json);
    }

    #[test]
    fn empty_invocation_is_normalized_to_help() {
        let args = normalize_args_for_parsing(vec![OsString::from("gh-download")]);

        assert_eq!(
            args,
            vec![OsString::from("gh-download"), OsString::from("--help")]
        );
    }
}
