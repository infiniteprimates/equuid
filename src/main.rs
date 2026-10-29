use std::{
    env,
    io::Write,
    ops::ControlFlow::{self, Break, Continue},
    process::ExitCode,
};

use clap::builder::StyledStr;
use equuid::cli::{self, CliError, CliRequest};

fn main() -> ExitCode {
    let cli_result = translate_cli_result(cli::build_request(env::args_os()));
    let cli_request = match cli_result {
        Break(mut cli_exit) => {
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

fn translate_cli_result(result: Result<CliRequest, CliError>) -> ControlFlow<CliExit, CliRequest> {
    match result {
        Ok(request) => {
            if let CliRequest::DisplayInfo(msg) = request {
                ControlFlow::Break(CliExit {
                    msg,
                    exit_code: ExitCode::SUCCESS,
                    stream: Box::new(anstream::stdout()),
                })
            } else {
                ControlFlow::Continue(request)
            }
        }
        Err(e) => ControlFlow::Break(CliExit {
            msg: e.to_styled_string(),
            exit_code: ExitCode::from(2),
            stream: Box::new(anstream::stderr()),
        }),
    }
}

pub struct CliExit {
    pub exit_code: ExitCode,
    pub msg: StyledStr,
    pub stream: Box<dyn Write>,
}
