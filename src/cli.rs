mod help;
mod resolve;
mod types;

use std::env;
use std::ffi::OsString;

use clap::FromArgMatches;

use crate::i18n::{Language, detect_language_from_args_and_env};

pub use self::help::{command, command_for_language};
pub use self::resolve::{
    pick_token, resolve_cli, resolve_debug, resolve_local_target, resolve_prefix_mode,
    resolve_proxy_base,
};
pub use self::types::{Cli, PrefixProxyMode, ResolvedOptions};

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

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use clap::Parser;

    use super::*;

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
    fn empty_invocation_is_normalized_to_help() {
        let args = normalize_args_for_parsing(vec![OsString::from("gh-download")]);

        assert_eq!(
            args,
            vec![OsString::from("gh-download"), OsString::from("--help")]
        );
    }
}
