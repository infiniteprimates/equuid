pub mod io;

mod native;
mod uuidgen;

use std::{
    ffi::OsString,
    io::Write,
    iter,
    path::{Path, PathBuf},
};

use crate::model::UuidRequest;

use anstyle::{AnsiColor, Effects, Style};
use clap::{
    Error as ClapError,
    builder::StyledStr,
    error::{ErrorKind, Result},
};
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
pub const DEFAULT_OUTPUTS: OutputTargets = OutputTargets {
    out: None,
    err: None,
};

pub fn parse(args_into_iter: impl IntoIterator<Item = OsString>) -> CliInvocation {
    let mut args_iter = args_into_iter.into_iter();

    let Some(program_path) = args_iter.next() else {
        return CliInvocation {
            personality: None,
            result: Err(CliError::ProgramName {
                msg: generate_error_styled_string("Missing program name."),
                kind: ProgramNameErrorKind::Arg0Missing,
            }),
            output_targets: DEFAULT_OUTPUTS,
        };
    };

    let Some(program_name) = get_program_name(&program_path) else {
        return CliInvocation {
            personality: None,
            result: Err(CliError::ProgramName {
                msg: generate_error_styled_string("Invalid program name format."),
                kind: ProgramNameErrorKind::Arg0UnknownFormat,
            }),
            output_targets: DEFAULT_OUTPUTS,
        };
    };

    let Some(personality) = CliPersonality::from_name(&program_name) else {
        return CliInvocation {
            personality: None,
            result: Err(CliError::ProgramName {
                msg: generate_error_styled_string(format!("Invalid program name {YELLOWBOLD_STYLE}{program_name}{RESET_STYLE} -- must be {BOLD_STYLE}equuid{RESET_STYLE} or {BOLD_STYLE}uuidgen{RESET_STYLE}.").as_str()),
                kind: ProgramNameErrorKind::Arg0InvalidName,
            }),
            output_targets: DEFAULT_OUTPUTS,
         };
    };

    let restored_iter = iter::once(OsString::from(&program_path)).chain(args_iter);

    let invocation = match personality {
        CliPersonality::Native => native::build_invocation(restored_iter),
        CliPersonality::Uuidgen => uuidgen::build_invocation(restored_iter),
    };

    translate_clap_error(invocation)
}

fn get_program_name(program_path: &OsString) -> Option<String> {
    Path::new(&program_path)
        .file_stem()
        .and_then(|os| os.to_str())
        .map(|s| s.to_string())
}

fn translate_clap_error(invocation: CliInvocation) -> CliInvocation {
    if let Err(CliError::ClapError(error)) = &invocation.result {
        match error.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => CliInvocation {
                personality: invocation.personality,
                result: Ok(UuidRequest::Info(error.render())),
                output_targets: invocation.output_targets,
            },
            _ => invocation,
        }
    } else {
        invocation
    }
}

fn generate_error_styled_string(msg: &str) -> StyledStr {
    let error_string = format!("{REDBOLD_STYLE}error:{RESET_STYLE} ");
    let mut error_styled = StyledStr::from(error_string);

    error_styled.push_str(msg);

    error_styled
}

pub struct CliInvocation {
    pub personality: Option<CliPersonality>,
    pub result: Result<UuidRequest, CliError>,
    pub output_targets: OutputTargets,
}

pub struct OutputTargets {
    pub out: Option<PathBuf>,
    pub err: Option<PathBuf>,
}

pub enum StreamKind {
    Out,
    Err,
}

pub struct Streams<'a> {
    pub out: &'a mut dyn Write,
    pub err: &'a mut dyn Write,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CliPersonality {
    Native,
    Uuidgen,
}

impl CliPersonality {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "equuid" => Some(Self::Native),
            "uuidgen" => Some(Self::Uuidgen),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProgramNameErrorKind {
    Arg0UnknownFormat,
    Arg0Missing,
    Arg0InvalidName,
}

#[derive(Debug, ThisError)]
pub enum CliError {
    #[error("{}", .0.render().ansi())]
    ClapError(#[from] ClapError),
    #[error("{}", msg.ansi())]
    ProgramName {
        msg: StyledStr,
        kind: ProgramNameErrorKind,
    },
    #[error("{}", .0.ansi())]
    ValidationError(StyledStr),
}

impl CliError {
    pub fn to_styled_string(&self) -> StyledStr {
        match self {
            CliError::ProgramName { msg, kind: _ } => msg.clone(),
            CliError::ClapError(e) => e.render(),
            CliError::ValidationError(msg) => msg.clone(),
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
    fn displayed(request: UuidRequest) -> String {
        match request {
            UuidRequest::Info(text) => text.to_string(),
            other => panic!("expected DisplayInfo, got {other:?}"),
        }
    }

    // Help and version are the two `ErrorKind`s that are answers rather than
    // failures. Everything else — including help requested *because* the
    // invocation was wrong — stays an `Err`.

    #[test]
    fn help_is_display_info_rather_than_an_error() {
        let invocation = parse(argv(&["equuid", "--help"]));
        let text = displayed(invocation.result.unwrap());
        assert!(
            text.contains("Usage: equuid"),
            "unexpected help text: {text}"
        );
        let Some(personality) = invocation.personality else {
            panic!("unexpected personality: None")
        };
        assert_eq!(
            personality,
            CliPersonality::Native,
            "unexpected personality {personality:?}",
        );
    }

    #[test]
    fn version_is_display_info_rather_than_an_error() {
        let invocation = parse(argv(&["equuid", "--version"]));
        let text = displayed(invocation.result.unwrap());
        // Read the version from the crate, not a literal: a release bump must not
        // turn this test red for the wrong reason.
        assert!(
            text.contains(env!("CARGO_PKG_VERSION")),
            "unexpected version text: {text}"
        );
        let Some(personality) = invocation.personality else {
            panic!("unexpected personality: None")
        };
        assert_eq!(
            personality,
            CliPersonality::Native,
            "unexpected personality {personality:?}",
        );
    }

    #[test]
    fn an_unknown_flag_is_a_clap_error() {
        let invocation = parse(argv(&["equuid", "--bogus"]));
        let error = invocation.result.unwrap_err();
        match error {
            CliError::ClapError(clap_error) => {
                assert_eq!(clap_error.kind(), ErrorKind::UnknownArgument)
            }
            other => panic!("expected ClapError, got {other:?}"),
        }
        let Some(personality) = invocation.personality else {
            panic!("unexpected personality: None")
        };
        assert_eq!(
            personality,
            CliPersonality::Native,
            "unexpected personality {personality:?}",
        );
    }

    #[test]
    fn an_unknown_program_name_names_the_program() {
        let invocation = parse(argv(&["notaprog", "--help"]));
        let error = invocation.result.unwrap_err();
        match error {
            CliError::ProgramName { msg, kind } => {
                assert!(
                    msg.to_string().contains("notaprog"),
                    "unexpected message: {msg}"
                );

                assert_eq!(kind, ProgramNameErrorKind::Arg0InvalidName);
            }
            other => panic!("expected ProgramName, got {other:?}"),
        }
        if let Some(personality) = invocation.personality {
            panic!("unexpected personality: {personality:?}")
        };
    }

    #[test]
    fn an_empty_argv_is_a_program_name_error() {
        let invocation = parse(argv(&[]));
        let error = invocation.result.unwrap_err();
        match error {
            CliError::ProgramName { msg: _, kind } => {
                assert!(
                    kind == ProgramNameErrorKind::Arg0Missing,
                    "unexpected kind: {kind:?}"
                );
            }
            other => panic!("expected ProgramName, got {other:?}"),
        }
        if let Some(personality) = invocation.personality {
            panic!("unexpected personality: {personality:?}")
        };
    }

    /// Dispatch is by the name the binary was invoked under, so each name must
    /// reach *its own* parser. `assert_ne!` is the load-bearing half: asserting
    /// only that both succeed would still pass if both names hit one parser.
    #[test]
    fn the_program_name_selects_the_parser() {
        let as_equuid = parse(argv(&["equuid", "--help"]));
        let as_uuidgen = parse(argv(&["uuidgen", "--help"]));

        let Some(personality) = as_equuid.personality else {
            panic!("unexpected equuid personality: None")
        };
        assert_eq!(
            personality,
            CliPersonality::Native,
            "unexpected equuid personality {personality:?}",
        );

        let Some(personality) = as_uuidgen.personality else {
            panic!("unexpected uuidgen personality: None")
        };
        assert_eq!(
            personality,
            CliPersonality::Uuidgen,
            "unexpected uuidgen personality {personality:?}",
        );

        let equuid_displayed = displayed(as_equuid.result.unwrap());
        let uuidgen_displayed = displayed(as_uuidgen.result.unwrap());
        assert_ne!(
            equuid_displayed, uuidgen_displayed,
            "both names rendered the same help"
        );
    }

    /// Pins today's placeholder on purpose: a recognised name with no arguments
    /// parses into a bare request and generates nothing yet. When generation
    /// lands this assertion changes, which is what makes it worth writing down.
    #[test]
    fn a_bare_invocation_is_a_v1_request() {
        assert!(matches!(
            parse(argv(&["equuid"])).result.unwrap(),
            UuidRequest::V1 {}
        ));
        assert!(matches!(
            parse(argv(&["uuidgen"])).result.unwrap(),
            UuidRequest::V1 {}
        ));
    }

    /// Dispatch reads the file *stem*, so a Windows binary name still reaches its
    /// own parser. Reverting to `file_name()` turns this red, which is the point.
    #[test]
    fn a_windows_style_program_name_dispatches_by_its_stem() {
        let invocation = parse(argv(&["equuid.exe", "--help"]));

        let Some(personality) = invocation.personality else {
            panic!("unexpected personality: None")
        };
        assert_eq!(
            personality,
            CliPersonality::Native,
            "unexpected personality {personality:?}",
        );
        displayed(invocation.result.unwrap());
    }

    /// Any single extension is stripped, not just `.exe` — a deliberate choice,
    /// not an accident of the API. Pinned so narrowing it later is a decision.
    ///
    /// Success path on purpose: dispatch is the subject, and an error result
    /// would blur "wrong parser dispatched" with "right parser rejected input".
    #[test]
    fn one_extension_is_stripped_whatever_it_is() {
        let invocation = parse(argv(&["uuidgen.sh"]));

        let Some(personality) = invocation.personality else {
            panic!("unexpected personality: None")
        };
        assert_eq!(
            personality,
            CliPersonality::Uuidgen,
            "unexpected personality {personality:?}",
        );
        assert!(
            matches!(invocation.result.unwrap(), UuidRequest::V1 {}),
            "expected the uuidgen parser to accept a bare invocation"
        );
    }
}
