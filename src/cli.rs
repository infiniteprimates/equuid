mod native;
mod uuidgen;

use std::{ffi::OsString, iter, path::Path};

use clap::{Error, builder::StyledStr, error::ErrorKind};
use thiserror::Error;

pub fn build_request(args_into_iter: impl IntoIterator<Item = impl Into<OsString> + Clone>) -> Result<CliRequest, CliError> {
    let mut args_iter = args_into_iter.into_iter();

    let Some(program_path) = args_iter.next().map(|i| i.into()) else {
        return Err(CliError::ProgramName(String::from("Unable to determine program name.")));
    };

    let Some(program_name) = get_program_name(&program_path) else {
        return Err(CliError::ProgramName(String::from("Unable to determine program name format.")));
    };

    let restored_iter = iter::once(OsString::from(&program_path)).chain(args_iter.map(|i| i.into()));

    let cli_request = match program_name.as_str() {
        "equuid" => native::build_request(restored_iter),
        "uuidgen" => uuidgen::build_request(restored_iter),
        actual_name => Err(CliError::ProgramName(format!("Unknown program name {actual_name}."))),
    }.or_else(|cli_err| {
        match cli_err {
            CliError::ClapError(clap_err) => handle_clap_err(clap_err),
            _ => Err(cli_err),
        }
    })?;

    Ok(cli_request)
}

fn get_program_name(program_path: &OsString) -> Option<String> {
    Path::new(&program_path)
        .file_name()
        .and_then(|os| os.to_str())
        .map(|s| s.to_string())
}

fn handle_clap_err(e: Error) -> Result<CliRequest, CliError> {
    match e.kind() {
        ErrorKind::DisplayHelp |
        ErrorKind::DisplayVersion |
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            Ok(CliRequest::DisplayInfo(e.render()))
        },
        _ => Err(CliError::ClapError(e)),
    }
}

#[derive(Debug)]
pub enum CliRequest {
    DisplayInfo(StyledStr),
    V1 {},
    V2 {},
    V3 {},
    V4 {},
    V5 {},
    V6 {},
    V7 {},
    V8 {},
    Dissect{},
}

fn clap_err_to_ansi(e: &clap::Error) -> String {
    e.render().ansi().to_string()
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Error processing program name: {0}")]
    ProgramName(String),
    #[error("Error processing arguments: {}", clap_err_to_ansi(.0))]
    ClapError(#[from] clap::Error),
}
