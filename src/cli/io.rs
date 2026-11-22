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
        }
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

#[cfg(test)]
mod tests {
    //! `render`'s contract, which is narrower than it looks: the `Result` it
    //! returns means "these bytes reached the stream", never "the outcome was
    //! good". The outcome is classified by the caller, before presentation.
    //!
    //! The `Ok(())` on the error arm is load-bearing — returning the outcome error
    //! from there is what made a usage error print twice, the second copy
    //! mislabelled `Failed to render output:`.

    use std::io;

    use anstream::AutoStream;

    use super::*;

    /// A writer that always fails, standing in for a full disk or a closed pipe.
    struct AlwaysFails;

    impl Write for AlwaysFails {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("writer is broken"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn into_string(stream: AutoStream<Vec<u8>>) -> String {
        String::from_utf8_lossy(&stream.into_inner()).into_owned()
    }

    /// Render with both streams captured. `AutoStream`s over buffers, so the
    /// captured text is stripped exactly as a pipe or file would receive it.
    fn capture(result: Result<UuidResponse>) -> (Result<()>, String, String) {
        let mut out = AutoStream::auto(Vec::new());
        let mut err = AutoStream::auto(Vec::new());

        let rendered = {
            let mut streams = Streams {
                out: &mut out,
                err: &mut err,
            };
            render(result, &None, &mut streams)
        };

        (rendered, into_string(out), into_string(err))
    }

    #[test]
    fn a_response_is_presented_on_stdout() {
        let (rendered, out, err) = capture(Ok(UuidResponse::Info(StyledStr::from("hello"))));

        assert!(rendered.is_ok(), "presenting a response should succeed");
        assert_eq!(out, "hello\n");
        assert!(err.is_empty(), "response wrote to stderr: {err}");
    }

    /// The placeholder renderer is `Debug` for now; pinning it here is what makes
    /// its replacement a deliberate change rather than a silent one.
    #[test]
    fn an_unhandled_response_falls_back_to_debug() {
        let (_, out, _) = capture(Ok(UuidResponse::V1 {}));

        assert_eq!(out, "V1\n");
    }

    #[test]
    fn an_error_is_presented_on_stderr() {
        let (rendered, out, err) = capture(Err(Error::msg("something went wrong")));

        assert!(
            rendered.is_ok(),
            "presenting an error is still a successful render"
        );
        assert!(out.is_empty(), "error wrote to stdout: {out}");
        assert!(
            err.contains("something went wrong"),
            "error is missing from stderr: {err}"
        );
    }

    /// "Rendered" and "succeeded" are different questions. This test is the one
    /// that keeps them apart: an outcome error renders cleanly and says so.
    #[test]
    fn reporting_an_error_returns_ok_not_the_error() {
        let (rendered, _, _) = capture(Err(Error::msg("outcome failure")));

        assert!(
            rendered.is_ok(),
            "render must not hand the outcome error back — that is what printed it twice"
        );
    }

    #[test]
    fn a_failed_write_is_reported_as_an_error() {
        let mut streams = Streams {
            out: &mut AlwaysFails,
            err: &mut AlwaysFails,
        };

        let rendered = render(
            Ok(UuidResponse::Info(StyledStr::from("hello"))),
            &None,
            &mut streams,
        );

        assert!(rendered.is_err(), "a failed write must not look successful");
    }

    #[test]
    fn a_failed_write_while_reporting_an_error_is_still_an_error() {
        let mut streams = Streams {
            out: &mut AlwaysFails,
            err: &mut AlwaysFails,
        };

        let rendered = render(Err(Error::msg("outcome failure")), &None, &mut streams);

        assert!(
            rendered.is_err(),
            "a failed write must not look successful even on the error path"
        );
    }
}
