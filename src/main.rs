use std::{env, process::ExitCode};

use equuid::cli::{self, CliRequest};

fn main() -> ExitCode {
    let cli_request = match cli::build_request(env::args_os()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    if let CliRequest::DisplayInfo(styled_string) = cli_request {
        anstream::println!("{}", styled_string.ansi());
        return ExitCode::SUCCESS;
    }

    println!("{cli_request:?}");

    ExitCode::SUCCESS
}
