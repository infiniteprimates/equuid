use std::ffi::OsString;

use clap::Parser;

use super::{CliError, UuidRequest};

pub fn build_request(args_iter: impl Iterator<Item = OsString>) -> Result<UuidRequest, CliError> {
    let _args = Args::try_parse_from(args_iter)?;

    Ok(UuidRequest::V1 {})
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "EqUUID - the thoroughbred of UUID generators.\nInvoked using the uuidgen compatibility layer."
)]
struct Args {}
