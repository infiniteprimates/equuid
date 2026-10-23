use std::ffi::OsString;

use clap::Parser;
use eyre::Result;

use super::CliRequest;

pub fn build_request(args_iter: impl Iterator<Item = impl Into<OsString> + Clone>) -> Result<CliRequest> {
    let _args = Args::try_parse_from(args_iter)?;

    eprintln!("Processing native CLI.");

    Ok(CliRequest::V1 {})
}

#[derive(Parser, Debug)]
#[command(version, about = "EqUUID, the thoroughbred of UUID generators.")]
struct Args {}
