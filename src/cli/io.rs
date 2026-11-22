use std::io::Write;

use super::{StreamKind, Streams};
use crate::{cli::CliPersonality, model::UuidResponse};

use anyhow::{Error, Result};
use clap::builder::StyledStr;

pub fn render(
    result: Result<UuidResponse>,
    _personality: &Option<CliPersonality>,
    streams: &mut Streams,
) -> Result<()> {
    // TODO: This is just a stub. The body of this will be completely replaced
    // as the CLI is implemented.
    match result {
        Ok(response) => emit(&render_response(&response), StreamKind::Out, streams),
        Err(error) => {
            emit(&render_error(&error), StreamKind::Err, streams)?;
            Ok(())
        },
    }
}

fn render_response(response: &UuidResponse) -> StyledStr {
    match response {
        UuidResponse::Info(msg) => msg.to_owned(),
        _ => StyledStr::from(format!("{response:?}")),
    }
}

fn render_error(error: &Error) -> StyledStr {
    StyledStr::from(format!("{error}"))
}

fn emit(output: &StyledStr, stream_kind: StreamKind, streams: &mut Streams) -> Result<()> {
    let stream: &mut dyn Write = match stream_kind {
        StreamKind::Out => streams.out,
        StreamKind::Err => streams.err,
    };

    writeln!(stream, "{}", output.ansi())?;
    stream.flush().map_err(Error::from)
}
