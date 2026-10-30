use std::{
    env,
    io::Write,
    ops::ControlFlow::{self, Break, Continue},
    process::ExitCode,
};

use clap::builder::StyledStr;
use equuid::cli::{self, CliError, CliRequest};

fn main() -> ExitCode {
    let mut stdout = anstream::stdout();
    let mut stderr = anstream::stderr();
    let cli_result =
        translate_cli_result(cli::build_request(env::args_os()), &mut stdout, &mut stderr);
    let cli_request = match cli_result {
        Break(cli_exit) => {
            writeln!(cli_exit.stream, "{}", cli_exit.msg.ansi()).unwrap_or_else(|err| {
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

fn translate_cli_result<'a>(
    result: Result<CliRequest, CliError>,
    stdout: &'a mut dyn Write,
    stderr: &'a mut dyn Write,
) -> ControlFlow<CliExit<'a>, CliRequest> {
    match result {
        Ok(request) => {
            if let CliRequest::DisplayInfo(msg) = request {
                ControlFlow::Break(CliExit {
                    msg,
                    exit_code: ExitCode::SUCCESS,
                    stream: stdout,
                })
            } else {
                ControlFlow::Continue(request)
            }
        }
        Err(e) => ControlFlow::Break(CliExit {
            msg: e.to_styled_string(),
            exit_code: ExitCode::from(2),
            stream: stderr,
        }),
    }
}

pub struct CliExit<'a> {
    pub exit_code: ExitCode,
    pub msg: StyledStr,
    pub stream: &'a mut dyn Write,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_streams() -> (Vec<u8>, Vec<u8>) {
        (Vec::new(), Vec::new())
    }

    #[test]
    fn ok_non_display_info_cli_result_translates_into_continue() {
        let (mut stdout, mut stderr) = create_streams();
        let cli_result = Ok(CliRequest::Dissect {});
        let Continue(cli_request) = translate_cli_result(cli_result, &mut stdout, &mut stderr)
        else {
            panic!("translate did not continue.")
        };

        assert!(matches!(cli_request, CliRequest::Dissect {}));
        assert!(stdout.is_empty(), "stdout was written to - {stdout:?}.");
        assert!(stderr.is_empty(), "stderr was written to - {stderr:?}");
    }

    #[test]
    fn ok_display_info_cli_result_translates_into_break_stdout_success() {
        let msg = "display info";
        let (mut stdout, mut stderr) = create_streams();
        let cli_result = Ok(CliRequest::DisplayInfo(StyledStr::from(msg)));
        let Break(cli_exit) = translate_cli_result(cli_result, &mut stdout, &mut stderr) else {
            panic!("translate did not break.")
        };

        assert_eq!(cli_exit.exit_code, ExitCode::SUCCESS, "invalid exit code",);
        assert_eq!(cli_exit.msg, StyledStr::from(msg), "invalid message",);
        assert!(stdout.is_empty(), "stdout was written to - {stdout:?}.");
        assert!(stderr.is_empty(), "stderr was written to - {stderr:?}");
        writeln!(cli_exit.stream, "stdout").expect("Write to stdout failed.");
        assert!(!stdout.is_empty(), "stdout was empty.");
        assert!(stderr.is_empty(), "stderr was written to - {stderr:?}");
    }

    #[test]
    fn err_cli_result_translates_into_break_stderr_failure() {
        let msg = "cli error";
        let (mut stdout, mut stderr) = create_streams();
        let cli_error = clap::Error::raw(
            clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
            msg,
        );
        let cli_result = Err(CliError::ClapError(cli_error));
        let Break(cli_exit) = translate_cli_result(cli_result, &mut stdout, &mut stderr) else {
            panic!("translate did not break.")
        };

        assert_eq!(cli_exit.exit_code, ExitCode::from(2), "invalid exit code",);
        assert_eq!(
            cli_exit.msg,
            StyledStr::from(format!("error: {}", msg)),
            "cli_err",
        );
        assert!(stdout.is_empty(), "stdout was written to - {stdout:?}.");
        assert!(stderr.is_empty(), "stderr was written to - {stderr:?}");
        writeln!(cli_exit.stream, "stderr").expect("Write to stderr failed.");
        assert!(!stderr.is_empty(), "stderr was empty.");
        assert!(stderr.is_empty(), "stderr was written to - {stderr:?}");
    }
}
