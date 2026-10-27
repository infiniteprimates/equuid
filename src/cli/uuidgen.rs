use std::ffi::OsString;

use clap::Parser;

use super::{CliError, CliRequest};

pub fn build_request(args_iter: impl Iterator<Item = impl Into<OsString> + Clone>) -> Result<CliRequest, CliError> {
    let _args = Args::try_parse_from(args_iter)?;

    Ok(CliRequest::V1 {})
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "The EqUUID implementation of uuidgen. EqUUID - the thoroughbred of UUID generators."
)]
struct Args {}
