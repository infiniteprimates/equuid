mod native;
mod uuidgen;

use std::{env::ArgsOs, path::Path};

use eyre::{Result, eyre};

pub fn build_request(args_os: ArgsOs) -> Result<CliRequest> {
    let Some(program_name) = get_program_name(args_os) else {
        return Err(eyre!("FATAL: Unable to determine process invocation."));
    };

    let cli_request = match program_name.as_str() {
        "equuid" => native::build_request(),
        "uuidgen" => uuidgen::build_request(),
        _ => Err(eyre!("FATAL: Unknown process invocation.")),
    }?;

    Ok(cli_request)
}

fn get_program_name(mut args_os: ArgsOs) -> Option<String> {
    let program_path = args_os.next()?;

    Path::new(&program_path)
        .file_name()
        .and_then(|os| os.to_str())
        .map(|s| s.to_string())
}

#[derive(Debug)]
pub enum CliRequest {
    V1 {},
    V2 {},
    V3 {},
    V4 {},
    V5 {},
    V6 {},
    V7 {},
    V8 {},
}
