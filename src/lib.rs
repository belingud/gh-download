mod cli;
mod download;
mod error;
mod i18n;
mod output;

pub use cli::{
    Cli, CliInvocation, PrefixProxyMode, ResolvedOptions, command, command_for_language,
    parse_cli_from_args, parse_cli_from_env, parse_cli_invocation_from_args,
    parse_cli_invocation_from_env, pick_token, resolve_cli, resolve_debug, resolve_local_target,
    resolve_prefix_mode, resolve_proxy_base,
};
pub use download::{
    DEFAULT_GH_PROXY, DEFAULT_GITHUB_API_BASE, DownloadStats, RunOutcome, Runner, RuntimeConfig,
    build_contents_api_url, choose_directory_target, format_remote_path, join_proxy_url,
    normalize_repo_path, relative_item_path,
};
pub use error::{AppError, UserFacingError, classify_error};
pub use i18n::{Language, detect_language_from_args_and_env};
pub use output::Output;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorContext {
    pub language: Language,
    pub token_present: bool,
}

pub fn resolve_error_context(invocation: &CliInvocation) -> ErrorContext {
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let gh_token = std::env::var("GH_TOKEN").ok();
    let config = crate::cli::load_active_config(invocation.config_path.as_deref())
        .ok()
        .flatten();
    let language = crate::cli::resolve_language(
        invocation.cli.language,
        config.as_ref().and_then(|value| value.lang),
        std::env::var("LC_ALL").ok().as_deref(),
        std::env::var("LC_MESSAGES").ok().as_deref(),
        std::env::var("LANG").ok().as_deref(),
    );
    let token_present = crate::cli::token_present(
        invocation.cli.token.as_deref(),
        config.as_ref().and_then(|value| value.token.as_deref()),
        github_token.as_deref(),
        gh_token.as_deref(),
    );

    ErrorContext {
        language,
        token_present,
    }
}

pub fn run_cli(cli: Cli) -> Result<RunOutcome, AppError> {
    let explicit_concurrency = cli.concurrency;
    run_cli_invocation(CliInvocation {
        cli,
        config_path: None,
        explicit_concurrency: Some(explicit_concurrency),
    })
}

pub fn run_cli_invocation(invocation: CliInvocation) -> Result<RunOutcome, AppError> {
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let gh_token = std::env::var("GH_TOKEN").ok();
    let config = crate::cli::load_active_config(invocation.config_path.as_deref())?;
    let token_source = crate::cli::debug_token_source_label(
        invocation.cli.token.as_deref(),
        config.as_ref().and_then(|value| value.token.as_deref()),
        github_token.as_deref(),
        gh_token.as_deref(),
    );
    let options = crate::cli::resolve_cli_with_config(
        invocation.cli,
        config,
        invocation.explicit_concurrency,
    )?;
    let output = if options.json {
        Output::new(!options.no_color, options.language).with_json_mode()
    } else {
        Output::new(!options.no_color, options.language)
    };
    output.startup(&options);
    if options.debug {
        output.debug_line(&format!("[debug] token-source: {}", token_source));
    }
    let runner = Runner::new(
        RuntimeConfig {
            api_base: options.api_base.clone(),
        },
        output.clone(),
    );
    let outcome = runner.run(&options)?;
    if options.json {
        output.print_json_success(&outcome.saved_path, &outcome.stats);
    }
    Ok(outcome)
}
