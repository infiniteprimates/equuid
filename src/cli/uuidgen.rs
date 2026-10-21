use clap::Parser;
use eyre::Result;

use super::CliRequest;

pub fn build_request() -> Result<CliRequest> {
    let _args = Args::parse();

    Ok(CliRequest::V1 {})
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "The EqUUID implementation of uuidgen. EqUUID - the thoroughbred of UUID generators."
)]
struct Args {}
