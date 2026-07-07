use std::io::IsTerminal as _;
use std::process::ExitCode;

use anstyle::{AnsiColor, Effects, Style};
use btpc_core::{Error, ErrorCategory};

use crate::command::CliColorPolicy;
use crate::output::redact_secrets;

const EXIT_INTERNAL: u8 = 1;
const EXIT_IO: u8 = 3;
const EXIT_DATA: u8 = 4;
const EXIT_UNSUPPORTED: u8 = 5;
const EXIT_VERIFICATION: u8 = 6;
const EXIT_CANCELLED: u8 = 130;

// Spec: CLI-EXIT-001
// Spec: CLI-DIAG-001
pub(crate) fn report(error: &Error, requested_color: Option<CliColorPolicy>) -> ExitCode {
    if error.category() != ErrorCategory::Verification {
        let diagnostic = Diagnostic::from_error(error);
        let color = use_color(requested_color);
        if color {
            let label = Style::new()
                .fg_color(Some(AnsiColor::Red.into()))
                .effects(Effects::BOLD);
            eprintln!(
                "{label}error{label:#} [{}]: {}",
                diagnostic.category, diagnostic.message
            );
        } else {
            eprintln!("error [{}]: {}", diagnostic.category, diagnostic.message);
        }
        if let Some(path) = diagnostic.path {
            eprintln!("  path: {}", crate::output::safe_path_display(path));
        }
        if let Some(field) = diagnostic.field {
            eprintln!("  field: {field}");
        }
        if let Some(offset) = diagnostic.offset {
            eprintln!("  byte offset: {offset}");
        }
        if let Some(hint) = diagnostic.hint {
            eprintln!("  hint: {hint}");
        }
    }
    ExitCode::from(exit_code(error))
}

struct Diagnostic<'a> {
    category: &'static str,
    message: String,
    path: Option<&'a std::path::Path>,
    field: Option<&'a str>,
    offset: Option<usize>,
    hint: Option<&'static str>,
}

impl<'a> Diagnostic<'a> {
    fn from_error(error: &'a Error) -> Self {
        Self {
            category: category_name(error.category()),
            message: redact_secrets(&error_message(error)),
            path: error.path(),
            field: error.field(),
            offset: error.offset(),
            hint: remediation(error),
        }
    }
}

fn error_message(error: &Error) -> String {
    match error {
        Error::Io { source, .. } => source.to_string(),
        Error::BencodeSyntax { message, .. }
        | Error::BencodeCanonical { message, .. }
        | Error::Metainfo { message, .. }
        | Error::Unsupported { message }
        | Error::Verification { message, .. } => message.clone(),
        Error::ResourceLimit {
            limit,
            actual,
            maximum,
        } => format!("resource limit {limit} exceeded: observed {actual}, maximum {maximum}"),
        Error::Cancelled => "operation cancelled".to_owned(),
        _ => error.to_string(),
    }
}

const fn category_name(category: ErrorCategory) -> &'static str {
    match category {
        ErrorCategory::Io => "io",
        ErrorCategory::BencodeSyntax => "bencode-syntax",
        ErrorCategory::BencodeCanonical => "bencode-canonical",
        ErrorCategory::Metainfo => "metainfo",
        ErrorCategory::ResourceLimit => "resource-limit",
        ErrorCategory::Unsupported => "unsupported",
        ErrorCategory::Verification => "verification",
        ErrorCategory::Cancelled => "cancelled",
        _ => "internal",
    }
}

fn remediation(error: &Error) -> Option<&'static str> {
    match error.category() {
        ErrorCategory::Io => Some("check that the path exists and is readable or writable"),
        ErrorCategory::BencodeSyntax | ErrorCategory::BencodeCanonical => {
            Some("validate the source metainfo or recreate it from the payload")
        }
        ErrorCategory::Metainfo => Some("check the named field and its related options"),
        ErrorCategory::ResourceLimit => {
            Some("raise the matching --max-* limit if the input is trusted")
        }
        ErrorCategory::Unsupported => Some("choose a supported mode or option combination"),
        ErrorCategory::Cancelled => Some("run the command again when ready"),
        _ => None,
    }
}

fn use_color(requested: Option<CliColorPolicy>) -> bool {
    match requested.unwrap_or_default() {
        CliColorPolicy::Always => true,
        CliColorPolicy::Never => false,
        CliColorPolicy::Auto => {
            std::env::var_os("NO_COLOR").is_none() && std::io::stderr().is_terminal()
        }
    }
}

pub(crate) fn suggestion<'a>(
    value: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    candidates
        .into_iter()
        .filter_map(|candidate| {
            let score = strsim::normalized_levenshtein(value, candidate);
            (score >= 0.72).then_some((candidate, score))
        })
        .max_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(candidate, _)| candidate)
}

const fn exit_code(error: &Error) -> u8 {
    match error.category() {
        ErrorCategory::Io => EXIT_IO,
        ErrorCategory::BencodeSyntax
        | ErrorCategory::BencodeCanonical
        | ErrorCategory::Metainfo
        | ErrorCategory::ResourceLimit => EXIT_DATA,
        ErrorCategory::Unsupported => EXIT_UNSUPPORTED,
        ErrorCategory::Verification => EXIT_VERIFICATION,
        ErrorCategory::Cancelled => EXIT_CANCELLED,
        _ => EXIT_INTERNAL,
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, suggestion};
    use btpc_core::Error;

    #[test]
    fn structured_diagnostics_include_context_and_redact_urls() {
        let error = Error::metainfo_field(
            "announce",
            "bad https://user:password@example/announce?passkey=secret",
        );
        let diagnostic = Diagnostic::from_error(&error);
        assert_eq!(diagnostic.category, "metainfo");
        assert_eq!(diagnostic.field, Some("announce"));
        assert!(diagnostic.message.contains("<redacted-url>"));
        assert!(!diagnostic.message.contains("password"));
        assert!(diagnostic.hint.is_some());
    }

    #[test]
    fn suggestions_are_conservative() {
        assert_eq!(
            suggestion("relese", ["release", "archive"]),
            Some("release")
        );
        assert_eq!(
            suggestion("completely-different", ["release", "archive"]),
            None
        );
    }
}
