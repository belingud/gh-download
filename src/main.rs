use std::process::ExitCode;

fn main() -> ExitCode {
    let invocation = gh_download::parse_cli_invocation_from_env();
    let no_color = invocation.cli.no_color;
    let json = invocation.cli.json;

    match gh_download::run_cli_invocation(invocation.clone()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            let context = gh_download::resolve_error_context(&invocation);
            let output = if json {
                gh_download::Output::new(!no_color, context.language).with_json_mode()
            } else {
                gh_download::Output::new(!no_color, context.language)
            };
            let user_error =
                gh_download::classify_error(&error, context.token_present, context.language);
            if json {
                output.print_json_error(&user_error);
            } else {
                output.print_user_error(&user_error);
            }
            ExitCode::from(1)
        }
    }
}
