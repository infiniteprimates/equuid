mod native;
mod uuidgen;

use std::{ffi::OsString, iter, path::Path};

use eyre::{Result, eyre};

pub fn build_request(args_into_iter: impl IntoIterator<Item = impl Into<OsString> + Clone>) -> Result<CliRequest> {
    let mut args_iter = args_into_iter.into_iter();

    let Some(program_path) = args_iter.next().map(|i| i.into()) else {
        return Err(eyre!("FATAL: Unable to determine program name."));
    };

    let Some(program_name) = get_program_name(&program_path) else {
        return Err(eyre!("FATAL: Unable to determine program name format."));
    };

    let restored_iter = iter::once(OsString::from(&program_path)).chain(args_iter.map(|i| i.into()));

    let cli_request = match program_name.as_str() {
        "equuid" => native::build_request(restored_iter),
        "uuidgen" => uuidgen::build_request(restored_iter),
        _ => Err(eyre!("FATAL: Unknown program name.")),
    }?;

    Ok(cli_request)
}

fn get_program_name(program_path: &OsString) -> Option<String> {
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
