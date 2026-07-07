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
    pub(crate) output: FilesystemPathJson,
    #[serde(rename = "output_display")]
    pub(crate) deprecated_output_display: String,
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
    pub(crate) nodes: Vec<DhtNodeJson>,
    pub(crate) source: Option<ByteStringJson>,
    pub(crate) comment: Option<ByteStringJson>,
    pub(crate) created_by: Option<ByteStringJson>,
    pub(crate) creation_date: Option<i64>,
    pub(crate) private: Option<bool>,
    pub(crate) canonical: bool,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DhtNodeJson {
    pub(crate) host: ByteStringJson,
    pub(crate) port: u16,
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
    pub(crate) path: FilesystemPathJson,
    #[serde(rename = "path_display")]
    pub(crate) deprecated_path_display: String,
    pub(crate) piece: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct FilesystemPathJson {
    pub(crate) schema: &'static str,
    pub(crate) display: String,
    pub(crate) encoding: &'static str,
    pub(crate) value: serde_json::Value,
}

pub(crate) fn filesystem_path_json(path: &std::path::Path) -> FilesystemPathJson {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt as _;
        FilesystemPathJson {
            schema: "btpc.filesystem-path.v2",
            display: safe_path_display(path),
            encoding: "unix-bytes-hex",
            value: serde_json::json!(encode_hex(path.as_os_str().as_bytes())),
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt as _;
        windows_path_json(
            safe_path_display(path),
            &path.as_os_str().encode_wide().collect::<Vec<_>>(),
        )
    }
}

pub(crate) fn safe_path_display(path: &std::path::Path) -> String {
    path.to_string_lossy()
        .chars()
        .flat_map(|character| {
            if character.is_control() {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

#[cfg(any(windows, test))]
fn windows_path_json(display: String, units: &[u16]) -> FilesystemPathJson {
    FilesystemPathJson {
        schema: "btpc.filesystem-path.v2",
        display,
        encoding: "windows-utf16",
        value: serde_json::json!(units),
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

pub(crate) fn stdout_path(path: &std::path::Path) -> Result<(), Error> {
    use std::io::Write as _;

    #[cfg(unix)]
    let bytes = {
        use std::os::unix::ffi::OsStrExt as _;
        let mut bytes = path.as_os_str().as_bytes().to_vec();
        bytes.push(b'\n');
        bytes
    };
    #[cfg(windows)]
    let bytes = {
        use std::os::windows::ffi::OsStrExt as _;
        windows_plain_path(path.as_os_str().encode_wide())
    };
    #[cfg(not(any(unix, windows)))]
    let bytes = format!("{}\n", safe_path_display(path)).into_bytes();

    std::io::stdout()
        .write_all(&bytes)
        .map_err(|source| Error::io("<stdout>", source))
}

#[cfg(any(windows, test))]
fn windows_plain_path(units: impl IntoIterator<Item = u16>) -> Vec<u8> {
    let mut output = b"windows-utf16:".to_vec();
    for (index, unit) in units.into_iter().enumerate() {
        if index > 0 {
            output.push(b',');
        }
        output.extend_from_slice(format!("{unit:04x}").as_bytes());
    }
    output.push(b'\n');
    output
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
    use super::{
        display_width, filesystem_path_json, redact_secrets, safe_path_display, windows_path_json,
        windows_plain_path,
    };

    #[cfg(unix)]
    #[test]
    fn filesystem_path_json_preserves_non_utf8_unix_bytes() {
        use std::os::unix::ffi::OsStrExt as _;

        let path = std::path::Path::new(std::ffi::OsStr::from_bytes(b"out-\xff"));
        let encoded = filesystem_path_json(path);
        assert_eq!(encoded.schema, "btpc.filesystem-path.v2");
        assert_eq!(encoded.encoding, "unix-bytes-hex");
        assert_eq!(encoded.value, serde_json::json!("6f75742dff"));
        assert!(encoded.display.contains('\u{fffd}'));

        let other = filesystem_path_json(std::path::Path::new(std::ffi::OsStr::from_bytes(
            b"out-\xfe",
        )));
        assert_eq!(encoded.display, other.display);
        assert_ne!(encoded.value, other.value);
    }

    #[test]
    fn filesystem_path_json_preserves_windows_utf16_code_units() {
        let encoded = windows_path_json("bad-�".to_owned(), &[98, 97, 100, 45, 0xd800]);
        assert_eq!(encoded.schema, "btpc.filesystem-path.v2");
        assert_eq!(encoded.encoding, "windows-utf16");
        assert_eq!(encoded.value, serde_json::json!([98, 97, 100, 45, 55_296]));
        assert_eq!(encoded.display, "bad-�");
    }

    #[test]
    fn path_display_escapes_terminal_control_characters() {
        assert_eq!(
            safe_path_display(std::path::Path::new("line\nbreak\tname")),
            "line\\nbreak\\tname"
        );
    }

    #[test]
    fn windows_plain_path_is_lossless_and_self_describing() {
        assert_eq!(
            windows_plain_path([98, 97, 100, 45, 0xd800]),
            b"windows-utf16:0062,0061,0064,002d,d800\n"
        );
    }

    #[test]
    fn widths_and_url_redaction_handle_unicode_and_punctuation() {
        assert_eq!(display_width("a界"), 3);
        assert_eq!(
            redact_secrets("failed for 'https://user:pass@example/announce?passkey=secret', retry"),
            "failed for '<redacted-url>', retry"
        );
    }
}
