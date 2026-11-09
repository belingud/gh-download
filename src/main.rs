use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use clap::error::ErrorKind;

fn main() -> ExitCode {
    match gh_download::try_parse_cli_invocation_from_env() {
        Ok(invocation) => run_download(invocation),
        Err(error) => handle_parse_error(error),
    }
}

fn run_download(invocation: gh_download::CliInvocation) -> ExitCode {
    let json = invocation.cli.json;

    match gh_download::run_cli_invocation(invocation.clone()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            let context = gh_download::resolve_error_context(&invocation);
            let user_error =
                gh_download::classify_error(&error, context.token_present, context.language);
            let output = if json {
                gh_download::Output::new(false, context.language).with_json_mode()
            } else {
                gh_download::Output::new(
                    gh_download::Output::color_enabled(invocation.cli.no_color),
                    context.language,
                )
            };
            if json {
                let details = gh_download::failure_details(&error);
                output.print_json_failure(&user_error, &details);
            } else {
                output.print_user_error(&user_error);
            }
            ExitCode::from(1)
        }
    }
}

fn handle_parse_error(error: clap::Error) -> ExitCode {
    match error.kind() {
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayVersion
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            let _ = error.print();
            ExitCode::SUCCESS
        }
        _ => {
            let args: Vec<OsString> = env::args_os().collect();
            let language = gh_download::detect_language_from_args_and_env(
                &args,
                env::var("LC_ALL").ok().as_deref(),
                env::var("LC_MESSAGES").ok().as_deref(),
                env::var("LANG").ok().as_deref(),
            );
            let json = flag_present(&args, "--json");
            if json {
                let user_error = gh_download::classify_parse_error(&error, language);
                let output = gh_download::Output::new(false, language).with_json_mode();
                output.print_json_error(&user_error);
            } else {
                let _ = error.print();
            }

            let code = u8::try_from(error.exit_code()).unwrap_or(2);
            ExitCode::from(code)
        }
    }
}

fn flag_present(args: &[OsString], flag: &str) -> bool {
    args.iter().skip(1).any(|arg| arg.to_string_lossy() == flag)
}
