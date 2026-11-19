use std::{
    env, ffi::OsString, io::{self, Write}, ops::ControlFlow::{self, Break, Continue}, process::ExitCode,
};

use equuid::cli::{self, CliError, CliInvocation, CliPersonality};
use equuid::model::{UuidRequest, UuidResponse};

use anyhow::Result;
use clap::builder::StyledStr;

const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    let args = env::args_os();
    let invocation = cli::parse(args);

    run(invocation)
}

fn run( invocation: CliInvocation) -> ExitCode {
    let personality = invocation.personality;
    let response = handle_request(invocation.result);

    render(response, personality, invocation.streams);

    exit_code(response, personality)
}

fn handle_request(request: std::result::Result<UuidRequest>) -> Result<UuidResponse> {
    // TODO: fully implemented in future PR.
    match invocation.result {
        Ok(request) => Ok(UuidResponse::V1 {}),
        Err(error) => error?
    }
}

fn exit_code(result: Result<UuidResponse>, personality: Option<CliPersonality>) -> ExitCode {
    match result {
        OK(_) => ExitCode::SUCCESS,
        Err(error) => {
            if let Some(cli_error) = error.downcast_ref::<CliError>() {
                if let Some(CliPersonality::Native) = personality {
                    return ExitCode::from(EXIT_USAGE);
                }
            }

            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_non_display_info_cli_result_translates_into_continue() {
        let cli_result = Ok(UuidRequest::Dissect {});
        let Continue(cli_request) = translate_cli_result(cli_result) else {
            panic!("translate did not continue.")
        };

        assert!(matches!(cli_request, UuidRequest::Dissect {}));
    }

    #[test]
    fn ok_display_info_cli_result_translates_into_break_stdout_success() {
        let msg = "display info";
        let cli_result = Ok(UuidRequest::Info(StyledStr::from(msg)));
        let Break(cli_exit) = translate_cli_result(cli_result) else {
            panic!("translate did not break.")
        };

        assert_eq!(cli_exit.exit_code, ExitCode::SUCCESS, "invalid exit code",);
        assert_eq!(cli_exit.msg, StyledStr::from(msg), "invalid message",);
        assert!(matches!(cli_exit.stream_kind, StreamKind::Out));
    }

    #[test]
    fn err_cli_result_translates_into_break_stderr_failure() {
        let msg = "cli error";
        let cli_error = clap::Error::raw(
            clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
            msg,
        );
        let cli_result = Err(CliError::ClapError(cli_error));
        let Break(cli_exit) = translate_cli_result(cli_result) else {
            panic!("translate did not break.")
        };

        assert_eq!(cli_exit.exit_code, ExitCode::from(2), "invalid exit code",);
        assert_eq!(
            cli_exit.msg,
            StyledStr::from(format!("error: {}", msg)),
            "cli_err",
        );
        assert!(matches!(cli_exit.stream_kind, StreamKind::Err));
    }
}
