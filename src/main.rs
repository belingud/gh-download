use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = gh_download::parse_cli_from_env();
    let no_color = cli.no_color;
    let language = gh_download::Language::detect(
        cli.language,
        std::env::var("LC_ALL").ok().as_deref(),
        std::env::var("LC_MESSAGES").ok().as_deref(),
        std::env::var("LANG").ok().as_deref(),
    );
    let token_present = cli.token.is_some()
        || std::env::var("GITHUB_TOKEN").ok().is_some()
        || std::env::var("GH_TOKEN").ok().is_some();

    match gh_download::run_cli(cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            let output = gh_download::Output::new(!no_color, language);
            let user_error = gh_download::classify_error(&error, token_present, language);
            output.print_user_error(&user_error);
            ExitCode::from(1)
        }
    }
}
