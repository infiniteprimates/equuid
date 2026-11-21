use std::{env, fs::File, io::BufWriter, path::PathBuf, process::ExitCode};

use equuid::cli::{self, CliError, CliInvocation, CliPersonality, Streams, io::render};
use equuid::model::{UuidRequest, UuidResponse};

use anyhow::Result;

const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    let args = env::args_os();
    let invocation = cli::parse(args);

    let mut out = match get_file_stream(invocation.output_targets.out.clone()) {
        Err(error) => {
            eprintln!("Failed to open output stream: {error}");
            return ExitCode::FAILURE;
        }
        Ok(out_opt) => out_opt,
    };

    let mut err = match get_file_stream(invocation.output_targets.err.clone()) {
        Err(error) => {
            eprintln!("Failed to open error stream: {error}");
            return ExitCode::FAILURE;
        }
        Ok(err_opt) => err_opt,
    };

    let streams = Streams {
        out: match out.as_mut() {
            Some(writer) => writer,
            None => &mut anstream::stdout(),
        },
        err: match err.as_mut() {
            Some(writer) => writer,
            None => &mut anstream::stderr(),
        },
    };

    run(invocation, streams)
}

fn run(invocation: CliInvocation, mut streams: Streams) -> ExitCode {
    let CliInvocation {
        personality,
        result,
        output_targets: _,
    } = invocation;
    let response = handle_request(result);

    if let Err(error) = render(&response, &personality, &mut streams) {
        eprintln!("Failed to render output: {error}");
        if let Some(_) = error.downcast_ref::<CliError>()
            && let Some(CliPersonality::Native) = personality
        {
            return ExitCode::from(EXIT_USAGE);
        }

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn handle_request(result: Result<UuidRequest, CliError>) -> Result<UuidResponse> {
    // TODO: fully implemented in future PR.
    let request = result?;

    match request {
        UuidRequest::Info(info) => Ok(UuidResponse::Info(info)),
        _ => Ok(UuidResponse::V1 {}),
    }
}

fn get_file_stream(path_buf: Option<PathBuf>) -> Result<Option<BufWriter<File>>> {
    match path_buf {
        Some(path) => Ok(Some(BufWriter::new(File::open(path)?))),
        None => Ok(None),
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
