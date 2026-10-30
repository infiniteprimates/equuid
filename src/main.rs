use std::{
    env,
    io::{self, Write},
    ops::ControlFlow::{self, Break, Continue},
    process::ExitCode,
};

use clap::builder::StyledStr;
use equuid::cli::{self, CliError, CliRequest};

fn main() -> ExitCode {
    let mut streams = Streams {
        out: &mut anstream::stdout(),
        err: &mut anstream::stderr(),
    };
    let cli_result = translate_cli_result(cli::build_request(env::args_os()));
    let cli_request = match cli_result {
        Break(cli_exit) => {
            write_output(&cli_exit.msg, cli_exit.stream_kind, &mut streams).unwrap_or_else(|err| {
                eprintln!("Unable to write to output stream. {err}");
                std::process::exit(2);
            });
            return cli_exit.exit_code;
        }
        Continue(r) => r,
    };

    println!("{cli_request:?}");
    ExitCode::SUCCESS
}

fn translate_cli_result(result: Result<CliRequest, CliError>) -> ControlFlow<CliExit, CliRequest> {
    match result {
        Ok(request) => {
            if let CliRequest::DisplayInfo(msg) = request {
                ControlFlow::Break(CliExit {
                    msg,
                    exit_code: ExitCode::SUCCESS,
                    stream_kind: StreamKind::Out,
                })
            } else {
                ControlFlow::Continue(request)
            }
        }
        Err(e) => ControlFlow::Break(CliExit {
            msg: e.to_styled_string(),
            exit_code: ExitCode::from(2),
            stream_kind: StreamKind::Err,
        }),
    }
}

fn write_output(
    output: &StyledStr,
    stream_kind: StreamKind,
    streams: &mut Streams,
) -> io::Result<()> {
    let stream: &mut dyn Write = match stream_kind {
        StreamKind::Out => streams.out,
        StreamKind::Err => streams.err,
    };

    writeln!(stream, "{}", output.ansi())
}

pub struct CliExit {
    pub exit_code: ExitCode,
    pub msg: StyledStr,
    pub stream_kind: StreamKind,
}

pub enum StreamKind {
    Out,
    Err,
}

pub struct Streams<'a> {
    out: &'a mut dyn Write,
    err: &'a mut dyn Write,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_non_display_info_cli_result_translates_into_continue() {
        let cli_result = Ok(CliRequest::Dissect {});
        let Continue(cli_request) = translate_cli_result(cli_result) else {
            panic!("translate did not continue.")
        };

        assert!(matches!(cli_request, CliRequest::Dissect {}));
    }

    #[test]
    fn ok_display_info_cli_result_translates_into_break_stdout_success() {
        let msg = "display info";
        let cli_result = Ok(CliRequest::DisplayInfo(StyledStr::from(msg)));
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
