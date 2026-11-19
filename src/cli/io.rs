use std::io::{Error, Write};

use super::{StreamKind, Streams};
use crate::{cli::CliPersonality, model::UuidResponse};

use anyhow::Result;
use clap::builder::StyledStr;

pub fn render(result: Result<UuidResponse>, _personality: Option<CliPersonality>, streams: &mut Streams) -> Result<(), Error>{
    // TODO: This is just a stub. The body of this will be completely replaced
    // as the CLI is implemented.
    match result {
        Ok(response) => emit(
            &StyledStr::from(format!("Render success: {response:?}")),
            StreamKind::Out,
            streams),
        Err(error) => emit(
            &StyledStr::from(format!("Render failure: {error}")),
            StreamKind::Err,
            streams),
    }
}

fn emit(
    output: &StyledStr,
    stream_kind: StreamKind,
    streams: &mut Streams,
) -> Result<(), Error> {
    let stream: &mut dyn Write = match stream_kind {
        StreamKind::Out => streams.out,
        StreamKind::Err => streams.err,
    };

    writeln!(stream, "{}", output.ansi())?;
    stream.flush()
}
