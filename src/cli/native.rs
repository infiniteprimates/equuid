use std::ffi::OsString;

use clap::Parser;

use super::{CliError, CliInvocation, CliPersonality, DEFAULT_OUTPUTS, UuidRequest};

pub fn build_invocation(args_iter: impl Iterator<Item = OsString>) -> CliInvocation {
    let parse_result = Args::try_parse_from(args_iter);
    if let Ok(_args) = parse_result {
        CliInvocation {
            personality: Some(CliPersonality::Native),
            result: Ok(UuidRequest::V1 {}),
            output_targets: DEFAULT_OUTPUTS,
        }
    } else {
        CliInvocation {
            personality: Some(CliPersonality::Native),
            result: Err(CliError::ClapError(parse_result.unwrap_err())),
            output_targets: DEFAULT_OUTPUTS,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "EqUUID, the thoroughbred of UUID generators.")]
struct Args {}
