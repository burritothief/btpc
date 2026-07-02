use btpc_core::Error;
use serde::Serialize;
use std::fmt::Display;
use unicode_width::UnicodeWidthStr as _;

// Spec: CLI-IO-001
// Spec: SEC-CONFIG-001
pub(crate) const REDACTED_URL: &str = "<redacted-url>";

// Spec: CLI-JSON-001
#[derive(Debug, Serialize)]
pub(crate) struct CreateJson<'a> {
    pub(crate) schema: &'static str,
    pub(crate) mode: &'static str,
    pub(crate) output: String,
    pub(crate) output_path: FilesystemPathJson,
    pub(crate) info_hash_v1: Option<String>,
    pub(crate) info_hash_v2: Option<String>,
    pub(crate) file_count: usize,
    pub(crate) payload_bytes: u64,
    pub(crate) piece_count: usize,
    pub(crate) piece_length: u64,
    pub(crate) piece_length_policy: Option<&'a str>,
    pub(crate) metrics_ms: MetricsJson,
}

#[derive(Debug, Serialize)]
pub(crate) struct MetricsJson {
    pub(crate) scan: u128,
    pub(crate) hash: u128,
    pub(crate) serialize: u128,
}

#[derive(Debug, Serialize)]
pub(crate) struct ByteStringJson {
    encoding: &'static str,
    value: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct InspectJson {
    pub(crate) schema: &'static str,
    pub(crate) mode: &'static str,
    pub(crate) name: ByteStringJson,
    pub(crate) total_bytes: u64,
    pub(crate) piece_length: u64,
    pub(crate) piece_count: u64,
    pub(crate) file_count: usize,
    pub(crate) info_hash_v1: Option<String>,
    pub(crate) info_hash_v2: Option<String>,
    pub(crate) trackers: Vec<Vec<ByteStringJson>>,
    pub(crate) web_seeds: Vec<ByteStringJson>,
    pub(crate) private: Option<bool>,
    pub(crate) canonical: bool,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ValidateJson {
    pub(crate) schema: &'static str,
    pub(crate) valid: bool,
    pub(crate) canonical: bool,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct VerifyJson {
    pub(crate) schema: &'static str,
    pub(crate) valid: bool,
    pub(crate) mismatches: Vec<VerifyMismatchJson>,
}

#[derive(Debug, Serialize)]
pub(crate) struct VerifyMismatchJson {
    pub(crate) kind: &'static str,
    pub(crate) path: String,
    pub(crate) path_exact: FilesystemPathJson,
    pub(crate) piece: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct FilesystemPathJson {
    pub(crate) display: String,
    pub(crate) encoding: &'static str,
    pub(crate) value: String,
}

pub(crate) fn filesystem_path_json(path: &std::path::Path) -> FilesystemPathJson {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt as _;
        FilesystemPathJson {
            display: path.to_string_lossy().into_owned(),
            encoding: "unix-bytes-hex",
            value: encode_hex(path.as_os_str().as_bytes()),
        }
    }
    #[cfg(not(unix))]
    {
        FilesystemPathJson {
            display: path.to_string_lossy().into_owned(),
            encoding: "utf-8",
            value: path.to_string_lossy().into_owned(),
        }
    }
}

pub(crate) fn write_json(value: &impl Serialize) -> Result<(), Error> {
    stdout_line(
        serde_json::to_string(value)
            .map_err(|error| Error::unsupported(format!("JSON encoding failed: {error}")))?,
    );
    Ok(())
}

pub(crate) fn write_json_pretty(value: &impl Serialize) -> Result<(), Error> {
    stdout_line(
        serde_json::to_string_pretty(value)
            .map_err(|error| Error::unsupported(format!("JSON encoding failed: {error}")))?,
    );
    Ok(())
}

pub(crate) fn stdout_line(value: impl Display) {
    println!("{value}");
}

pub(crate) fn stdout_text(value: impl Display) {
    print!("{value}");
}

pub(crate) fn stderr_line(value: impl Display) {
    eprintln!("{value}");
}

pub(crate) fn byte_string_json(bytes: &[u8]) -> ByteStringJson {
    match std::str::from_utf8(bytes) {
        Ok(text) => ByteStringJson {
            encoding: "utf-8",
            value: text.to_owned(),
        },
        Err(_) => ByteStringJson {
            encoding: "hex",
            value: encode_hex(bytes),
        },
    }
}

pub(crate) fn redacted_url_json(_bytes: &[u8]) -> ByteStringJson {
    ByteStringJson {
        encoding: "utf-8",
        value: REDACTED_URL.to_owned(),
    }
}

pub(crate) fn display_bytes(bytes: &[u8]) -> String {
    std::str::from_utf8(bytes)
        .map_or_else(|_| format!("0x{}", encode_hex(bytes)), ToOwned::to_owned)
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut output, byte| {
        write!(output, "{byte:02x}").expect("writing to String cannot fail");
        output
    })
}

pub(crate) fn encode_hex_public(bytes: &[u8]) -> String {
    encode_hex(bytes)
}

pub(crate) fn display_width(value: &str) -> usize {
    value.width()
}

pub(crate) fn redact_secrets(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut remainder = value;
    while let Some(scheme_end) = remainder.find("://") {
        let prefix = &remainder[..scheme_end];
        let start = prefix
            .rfind(|character: char| character.is_whitespace() || "\"'=<([{`".contains(character))
            .map_or(0, |index| index + 1);
        output.push_str(&remainder[..start]);
        output.push_str(REDACTED_URL);
        let url = &remainder[start..];
        let end = url
            .find(|character: char| character.is_whitespace() || "\"'<>)]},`".contains(character))
            .unwrap_or(url.len());
        remainder = &url[end..];
    }
    output.push_str(remainder);
    output
}

#[cfg(test)]
mod tests {
    use super::{display_width, redact_secrets};

    #[test]
    fn widths_and_url_redaction_handle_unicode_and_punctuation() {
        assert_eq!(display_width("a界"), 3);
        assert_eq!(
            redact_secrets("failed for 'https://user:pass@example/announce?passkey=secret', retry"),
            "failed for '<redacted-url>', retry"
        );
    }
}
