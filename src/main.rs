use std::io::BufWriter;
use std::{env, fs::File, io::Write, path::PathBuf, process::ExitCode};

use anstream::AutoStream;
use equuid::cli::{self, CliError, CliInvocation, CliPersonality, Streams, io::render};
use equuid::model::{UuidRequest, UuidResponse};

use anyhow::Result;

const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    let args = env::args_os();
    let invocation = cli::parse(args);

    let mut out = match get_file_stream(invocation.output_targets.out.clone()) {
        Err(error) => {
            eprintln!("Failed to open output stream: {error}");
            return ExitCode::FAILURE;
        }
        Ok(out_opt) => out_opt,
    };

    let mut err = match get_file_stream(invocation.output_targets.err.clone()) {
        Err(error) => {
            eprintln!("Failed to open error stream: {error}");
            return ExitCode::FAILURE;
        }
        Ok(err_opt) => err_opt,
    };

    let streams = Streams {
        out: match out.as_mut() {
            Some(writer) => writer,
            None => &mut anstream::stdout(),
        },
        err: match err.as_mut() {
            Some(writer) => writer,
            None => &mut anstream::stderr(),
        },
    };

    run(invocation, streams)
}

fn run(invocation: CliInvocation, mut streams: Streams) -> ExitCode {
    let CliInvocation {
        personality,
        result,
        output_targets: _,
    } = invocation;
    let handle_response = handle_request(result);

    let is_error = handle_response.as_ref().is_err();
    let is_cli_error = handle_response
        .as_ref()
        .is_err_and(|e| e.downcast_ref::<CliError>().is_some());

    if let Err(error) = render(handle_response, &personality, &mut streams) {
        writeln!(anstream::stderr(), "Failed to render output: {error}").unwrap();
    }

    if is_error {
        if let Some(CliPersonality::Native) = personality
            && is_cli_error
        {
            return ExitCode::from(EXIT_USAGE);
        }

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn handle_request(result: Result<UuidRequest, CliError>) -> Result<UuidResponse> {
    // TODO: fully implemented in future PR.
    let request = result?;

    match request {
        UuidRequest::Info(info) => Ok(UuidResponse::Info(info)),
        _ => Ok(UuidResponse::V1 {}),
    }
}

fn get_file_stream(path_buf: Option<PathBuf>) -> Result<Option<Box<dyn Write>>> {
    match path_buf {
        Some(path) => {
            let file = File::create(path)?;
            let stream = AutoStream::auto(file);
            let buffered = BufWriter::new(stream);
            Ok(Some(Box::new(buffered)))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    //! `run`'s policy: which exit code, and which stream.
    //!
    //! Everything here drives `run` directly with in-memory streams. Dispatch is
    //! `cli`'s contract and presentation is `io`'s; testing them again through
    //! `main` would couple this module to both.

    use std::{fs, io::ErrorKind, path::PathBuf};

    use clap::error::ErrorKind as ClapErrorKind;

    use super::*;

    /// Run one invocation, capturing both streams.
    ///
    /// The streams are `AutoStream`s over in-memory buffers for the same reason
    /// production uses them: stripping is the stream's job, not the renderer's.
    /// A bare `Vec<u8>` would receive raw escapes and every plain-text assertion
    /// below would read interleaved `ESC` sequences.
    fn run_captured(invocation: CliInvocation) -> (ExitCode, String, String) {
        let mut out = AutoStream::auto(Vec::new());
        let mut err = AutoStream::auto(Vec::new());

        let code = {
            let streams = Streams {
                out: &mut out,
                err: &mut err,
            };
            run(invocation, streams)
        };

        (
            code,
            String::from_utf8_lossy(&out.into_inner()).into_owned(),
            String::from_utf8_lossy(&err.into_inner()).into_owned(),
        )
    }

    fn invocation(
        personality: Option<CliPersonality>,
        result: Result<UuidRequest, CliError>,
    ) -> CliInvocation {
        CliInvocation {
            personality,
            result,
            output_targets: cli::DEFAULT_OUTPUTS,
        }
    }

    fn clap_error(kind: ClapErrorKind, msg: &str) -> CliError {
        CliError::ClapError(clap::Error::raw(kind, msg))
    }

    // --- exit codes -------------------------------------------------------
    //
    // Three-way by design: equuid's usage errors exit 2 (util-linux parity is
    // uuidgen's 1), a bad program name exits 1, success exits 0. Writing these
    // down is what stops a refactor from changing the contract by accident.

    #[test]
    fn a_native_usage_error_exits_two() {
        let (code, out, err) = run_captured(invocation(
            Some(CliPersonality::Native),
            Err(clap_error(
                ClapErrorKind::UnknownArgument,
                "unexpected argument",
            )),
        ));

        assert_eq!(code, ExitCode::from(EXIT_USAGE), "unexpected exit code");
        assert!(out.is_empty(), "usage error wrote to stdout: {out}");
        assert!(err.contains("unexpected argument"), "no message: {err}");
    }

    #[test]
    fn a_uuidgen_usage_error_exits_one() {
        let (code, out, err) = run_captured(invocation(
            Some(CliPersonality::Uuidgen),
            Err(clap_error(
                ClapErrorKind::UnknownArgument,
                "unexpected argument",
            )),
        ));

        assert_eq!(code, ExitCode::FAILURE, "unexpected exit code");
        assert!(out.is_empty(), "usage error wrote to stdout: {out}");
        assert!(err.contains("unexpected argument"), "no message: {err}");
    }

    /// No personality means dispatch never happened, so there is no surface
    /// whose usage convention applies: a program-name error exits 1 whatever the
    /// name would have been.
    #[test]
    fn a_program_name_error_exits_one() {
        let error = cli::parse([std::ffi::OsString::from("notaprog"), "help".into()]);
        let (code, out, err) = run_captured(error);

        assert_eq!(code, ExitCode::FAILURE, "unexpected exit code");
        assert!(out.is_empty(), "program-name error wrote to stdout: {out}");
        assert!(err.contains("Invalid program name"), "no message: {err}");
    }

    #[test]
    fn success_exits_zero_and_writes_to_stdout_only() {
        let (code, out, err) = run_captured(invocation(
            Some(CliPersonality::Native),
            Ok(UuidRequest::V1 {}),
        ));

        assert_eq!(code, ExitCode::SUCCESS, "unexpected exit code");
        assert_eq!(out, "V1\n");
        assert!(err.is_empty(), "success wrote to stderr: {err}");
    }

    #[test]
    fn help_exits_zero_and_writes_to_stdout_only() {
        let (code, out, err) = run_captured(cli::parse([
            std::ffi::OsString::from("equuid"),
            "--help".into(),
        ]));

        assert_eq!(code, ExitCode::SUCCESS, "unexpected exit code");
        assert!(out.contains("Usage: equuid"), "no help on stdout: {out}");
        assert!(err.is_empty(), "help wrote to stderr: {err}");
    }

    // --- the double print -------------------------------------------------
    //
    // The regression these pin: `render` used to hand the outcome error back, so
    // the caller printed it a second time under a "Failed to render output"
    // prefix that was false, and raw ANSI leaked because that copy bypassed
    // anstream.

    #[test]
    fn a_usage_error_is_rendered_once() {
        let (_, _, err) = run_captured(invocation(
            Some(CliPersonality::Native),
            Err(clap_error(
                ClapErrorKind::UnknownArgument,
                "unexpected argument",
            )),
        ));

        assert_eq!(
            err.matches("unexpected argument").count(),
            1,
            "error rendered more than once: {err}"
        );
        assert!(
            !err.contains("Failed to render output"),
            "an outcome error was reported as a render failure: {err}"
        );
    }

    // --- file targets -----------------------------------------------------
    //
    // The flag that would set `output_targets` does not exist yet, so this is the
    // only coverage of the file path: it must strip styles the same way a shell
    // redirect does, because `AutoStream::auto` is what decides that.

    #[test]
    fn a_file_target_receives_unstyled_bytes() {
        let dir = std::env::temp_dir();
        let stamp = format!("{}-{}", std::process::id(), line!());
        let out_path = dir.join(format!("equuid-out-{stamp}.txt"));
        let err_path = dir.join(format!("equuid-err-{stamp}.txt"));

        // Two writers, not one shared: `Streams` holds both mutably at once, and
        // out/err are separate targets in the real code path anyway.
        let mut out = get_file_stream(Some(out_path.clone()))
            .expect("opening a fresh temp path should succeed")
            .expect("a path was supplied, so a writer is expected");
        let mut err = get_file_stream(Some(err_path.clone()))
            .expect("opening a fresh temp path should succeed")
            .expect("a path was supplied, so a writer is expected");

        {
            let mut streams = Streams {
                out: &mut out,
                err: &mut err,
            };
            render(
                Err(clap_error(ClapErrorKind::UnknownArgument, "unexpected argument").into()),
                &None,
                &mut streams,
            )
            .expect("rendering into a fresh file should succeed");
        }

        drop(out);
        drop(err);

        let out_bytes = fs::read(&out_path).expect("out file should be readable");
        let err_bytes = fs::read(&err_path).expect("err file should be readable");
        fs::remove_file(&out_path).ok();
        fs::remove_file(&err_path).ok();

        assert!(
            !err_bytes.contains(&0x1b),
            "file target received ANSI escapes: {}",
            String::from_utf8_lossy(&err_bytes)
        );
        assert!(
            String::from_utf8_lossy(&err_bytes).contains("unexpected argument"),
            "error file is missing the message"
        );
        assert!(out_bytes.is_empty(), "an error wrote to the out file");
    }

    #[test]
    fn a_missing_directory_is_reported_as_an_open_failure() {
        let path = PathBuf::from("/equuid-nonexistent-dir-xyz/out.txt");

        let error = match get_file_stream(Some(path)) {
            Ok(_) => panic!("a path in a missing directory should not open"),
            Err(error) => error,
        };
        let io_error = error
            .downcast_ref::<std::io::Error>()
            .expect("file errors should stay io::Error through anyhow");

        assert!(
            matches!(
                io_error.kind(),
                ErrorKind::NotFound | ErrorKind::PermissionDenied
            ),
            "unexpected error kind: {io_error:?}"
        );
    }
}
