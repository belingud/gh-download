mod cli;
mod download;
mod error;
mod i18n;
mod output;

pub use cli::{
    Cli, PrefixProxyMode, ResolvedOptions, command, command_for_language, parse_cli_from_args,
    parse_cli_from_env, pick_token, resolve_cli, resolve_debug, resolve_local_target,
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

pub fn run_cli(cli: Cli) -> Result<RunOutcome, AppError> {
    let options = resolve_cli(cli)?;
    let output = Output::new(!options.no_color, options.language);
    output.startup(&options);
    let runner = Runner::new(RuntimeConfig::default(), output);
    runner.run(&options)
}
