use std::{env, process::ExitCode};

use equuid::cli;

fn main() -> ExitCode {
    let cli_request = match cli::build_request(env::args_os()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e:?}");
            return ExitCode::FAILURE;
        }
    };

    println!("CliRequest:");
    println!("{cli_request:?}");

    println!("This is were we would generate a uuid.");

    ExitCode::SUCCESS
}
