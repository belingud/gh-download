use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = gh_download::parse_cli_from_env();
    let no_color = cli.no_color;
    let json = cli.json;
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
            let output = if json {
                gh_download::Output::new(!no_color, language).with_json_mode()
            } else {
                gh_download::Output::new(!no_color, language)
            };
            let user_error = gh_download::classify_error(&error, token_present, language);
            if json {
                output.print_json_error(&user_error);
            } else {
                output.print_user_error(&user_error);
            }
            ExitCode::from(1)
        }
    }
}
