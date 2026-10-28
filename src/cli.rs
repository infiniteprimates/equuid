mod native;
mod uuidgen;

use std::{ffi::OsString, iter, path::Path};

use anstyle::{AnsiColor, Effects, Style};
use clap::{Error as ClapError, builder::StyledStr, error::ErrorKind};
use thiserror::Error as ThisError;

pub const BOLD_STYLE: Style = Style::new().effects(Effects::BOLD);
pub const GREENBOLD_STYLE: Style = Style::new()
    .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Green)))
    .effects(Effects::BOLD);
pub const YELLOWBOLD_STYLE: Style = Style::new()
    .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Yellow)))
    .effects(Effects::BOLD);
pub const REDBOLD_STYLE: Style = Style::new()
    .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Red)))
    .effects(Effects::BOLD);
pub const RESET_STYLE: anstyle::Reset = anstyle::Reset;

pub fn build_request(
    args_into_iter: impl IntoIterator<Item = impl Into<OsString> + Clone>,
) -> Result<CliRequest, CliError> {
    let mut args_iter = args_into_iter.into_iter();

    let Some(program_path) = args_iter.next().map(|i| i.into()) else {
        return Err(CliError::ProgramName {
            msg: generate_error_styled_string("Missing program name."),
            kind: ProgramNameErrorKind::Arg0Missing,
        });
    };

    let Some(program_name) = get_program_name(&program_path) else {
        return Err(CliError::ProgramName {
            msg: generate_error_styled_string("Invalid program name format."),
            kind: ProgramNameErrorKind::Arg0UnknownFormat,
        });
    };

    let restored_iter =
        iter::once(OsString::from(&program_path)).chain(args_iter.map(|i| i.into()));

    let cli_request = match program_name.as_str() {
        "equuid" => native::build_request(restored_iter),
        "uuidgen" => uuidgen::build_request(restored_iter),
        actual_name => Err(CliError::ProgramName {
            msg: generate_error_styled_string(format!("Invalid program name {YELLOWBOLD_STYLE}{actual_name}{RESET_STYLE} -- must be {BOLD_STYLE}equuid{RESET_STYLE} or {BOLD_STYLE}uuidgen{RESET_STYLE}.").as_str()),
            kind: ProgramNameErrorKind::Arg0InvalidName,
        }),
    }
    .or_else(|cli_err| match cli_err {
        CliError::ClapError(clap_err) => handle_clap_err(clap_err),
        _ => Err(cli_err),
    })?;

    Ok(cli_request)
}

fn get_program_name(program_path: &OsString) -> Option<String> {
    Path::new(&program_path)
        .file_name()
        .and_then(|os| os.to_str())
        .map(|s| s.to_string())
}

fn handle_clap_err(e: ClapError) -> Result<CliRequest, CliError> {
    match e.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            Ok(CliRequest::DisplayInfo(e.render()))
        }
        _ => Err(CliError::ClapError(e)),
    }
}

fn generate_error_styled_string(msg: &str) -> StyledStr {
    let error_string = format!("{REDBOLD_STYLE}error:{RESET_STYLE} ");
    let mut error_styled = StyledStr::from(error_string);

    error_styled.push_str(msg);

    error_styled
}

#[derive(Debug)]
pub enum CliRequest {
    DisplayInfo(StyledStr),
    V1 {},
    V2 {},
    V3 {},
    V4 {},
    V5 {},
    V6 {},
    V7 {},
    V8 {},
    Dissect {},
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProgramNameErrorKind {
    Arg0UnknownFormat,
    Arg0Missing,
    Arg0InvalidName,
}

#[derive(Debug, ThisError)]
pub enum CliError {
    #[error("Error processing program name: {}", msg.to_string())]
    ProgramName {
        msg: StyledStr,
        kind: ProgramNameErrorKind,
    },
    #[error("Error processing arguments: {}", .0.to_string())]
    ClapError(#[from] ClapError),
}

impl CliError {
    pub fn to_styled_string(&self) -> StyledStr {
        match self {
            CliError::ProgramName { msg, kind: _ } => msg.clone(),
            CliError::ClapError(e) => e.render(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dispatcher's contract, asserted on the *request* rather than on the
    /// process: which arm comes out, for this input. Printing and exit codes live
    /// in `main`, so none of these change when that policy changes.
    fn argv(items: &[&str]) -> Vec<OsString> {
        items.iter().map(|item| OsString::from(*item)).collect()
    }

    /// `DisplayInfo` carries styled text instead of a pre-rendered `String` — the
    /// caller decides what colour its stream deserves. Assert against the *plain*
    /// rendering: `Display` strips styling, so `contains("Usage:")` matches. The
    /// styled form would interleave escape sequences inside the very words being
    /// matched, which is a property of the printer, not of the request.
    fn displayed(request: CliRequest) -> String {
        match request {
            CliRequest::DisplayInfo(text) => text.to_string(),
            other => panic!("expected DisplayInfo, got {other:?}"),
        }
    }

    // Help and version are the two `ErrorKind`s that are answers rather than
    // failures. Everything else — including help requested *because* the
    // invocation was wrong — stays an `Err`.

    #[test]
    fn help_is_display_info_rather_than_an_error() {
        let text = displayed(build_request(argv(&["equuid", "--help"])).unwrap());
        assert!(
            text.contains("Usage: equuid"),
            "unexpected help text: {text}"
        );
    }

    #[test]
    fn version_is_display_info_rather_than_an_error() {
        let text = displayed(build_request(argv(&["equuid", "--version"])).unwrap());
        // Read the version from the crate, not a literal: a release bump must not
        // turn this test red for the wrong reason.
        assert!(
            text.contains(env!("CARGO_PKG_VERSION")),
            "unexpected version text: {text}"
        );
    }

    #[test]
    fn an_unknown_flag_is_a_clap_error() {
        let error = build_request(argv(&["equuid", "--bogus"])).unwrap_err();
        match error {
            CliError::ClapError(clap_error) => {
                assert_eq!(clap_error.kind(), ErrorKind::UnknownArgument)
            }
            other => panic!("expected ClapError, got {other:?}"),
        }
    }

    #[test]
    fn an_unknown_program_name_names_the_program() {
        let error = build_request(argv(&["notaprog", "--help"])).unwrap_err();
        match error {
            CliError::ProgramName { msg, kind } => {
                assert!(
                    msg.to_string().contains("notaprog"),
                    "unexpected message: {msg}"
                );

                assert!(
                    kind == ProgramNameErrorKind::Arg0InvalidName,
                    "unexpected kind: {kind:?}"
                );
            }
            other => panic!("expected ProgramName, got {other:?}"),
        }
    }

    #[test]
    fn an_empty_argv_is_a_program_name_error() {
        let error = build_request(argv(&[])).unwrap_err();
        match error {
            CliError::ProgramName { msg: _, kind } => {
                assert!(
                    kind == ProgramNameErrorKind::Arg0Missing,
                    "unexpected kind: {kind:?}"
                );
            }
            other => panic!("expected ProgramName, got {other:?}"),
        }
    }

    /// Dispatch is by the name the binary was invoked under, so each name must
    /// reach *its own* parser. `assert_ne!` is the load-bearing half: asserting
    /// only that both succeed would still pass if both names hit one parser.
    #[test]
    fn the_program_name_selects_the_parser() {
        let as_equuid = displayed(build_request(argv(&["equuid", "--help"])).unwrap());
        let as_uuidgen = displayed(build_request(argv(&["uuidgen", "--help"])).unwrap());

        assert!(
            as_uuidgen.contains("uuidgen"),
            "uuidgen help did not come from its own parser: {as_uuidgen}"
        );
        assert_ne!(as_equuid, as_uuidgen, "both names rendered the same help");
    }

    /// Pins today's placeholder on purpose: a recognised name with no arguments
    /// parses into a bare request and generates nothing yet. When generation
    /// lands this assertion changes, which is what makes it worth writing down.
    #[test]
    fn a_bare_invocation_is_a_v1_request() {
        assert!(matches!(
            build_request(argv(&["equuid"])).unwrap(),
            CliRequest::V1 {}
        ));
        assert!(matches!(
            build_request(argv(&["uuidgen"])).unwrap(),
            CliRequest::V1 {}
        ));
    }
}
