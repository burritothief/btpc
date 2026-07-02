use btpc_core::create::{PieceLengthMode, validate_piece_length};

use super::{CreationDateValue, EntropyValue};

pub(super) fn parse_piece_length(value: &str) -> Result<u64, String> {
    let piece_length = if let Some(exponent) = value.strip_prefix("2^") {
        let exponent = exponent
            .parse::<u32>()
            .map_err(|_| "invalid exponent".to_owned())?;
        1_u64
            .checked_shl(exponent)
            .ok_or_else(|| "piece length overflow".to_owned())?
    } else if let Some(number) = value.strip_suffix("MiB") {
        number
            .parse::<u64>()
            .map_err(|_| "invalid MiB value".to_owned())?
            .checked_mul(1024 * 1024)
            .ok_or_else(|| "piece length overflow".to_owned())?
    } else if let Some(number) = value.strip_suffix("KiB") {
        number
            .parse::<u64>()
            .map_err(|_| "invalid KiB value".to_owned())?
            .checked_mul(1024)
            .ok_or_else(|| "piece length overflow".to_owned())?
    } else {
        value
            .parse::<u64>()
            .map_err(|_| "piece length must be bytes, KiB, MiB, or 2^N".to_owned())?
    };
    validate_piece_length(piece_length, PieceLengthMode::V1).map_err(|error| error.to_string())
}

pub(super) fn parse_creation_date(value: &str) -> Result<CreationDateValue, String> {
    match value {
        "none" => Ok(CreationDateValue::None),
        "now" => Ok(CreationDateValue::Timestamp(chrono::Utc::now().timestamp())),
        _ => value
            .parse::<i64>()
            .map(CreationDateValue::Timestamp)
            .or_else(|_| {
                chrono::DateTime::parse_from_rfc3339(value)
                    .map(|value| CreationDateValue::Timestamp(value.timestamp()))
                    .map_err(|_| "expected now, none, UNIX seconds, or RFC3339".to_owned())
            }),
    }
}

pub(super) fn parse_entropy(value: &str) -> Result<EntropyValue, String> {
    match value {
        "none" => Ok(EntropyValue::None),
        "random" => Ok(EntropyValue::Random),
        _ => {
            if value.len() % 2 != 0 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err("entropy must be random, none, or even-length HEX".to_owned());
            }
            (0..value.len())
                .step_by(2)
                .map(|index| {
                    u8::from_str_radix(&value[index..index + 2], 16)
                        .map_err(|_| "invalid entropy hex".to_owned())
                })
                .collect::<Result<Vec<_>, _>>()
                .map(EntropyValue::Exact)
        }
    }
}

pub(super) fn parse_node(value: &str) -> Result<(Vec<u8>, u16), String> {
    let (host, port) = value
        .rsplit_once(':')
        .ok_or_else(|| "node must use HOST:PORT".to_owned())?;
    if host.is_empty() {
        return Err("node host must not be empty".to_owned());
    }
    let port = port
        .parse::<u16>()
        .map_err(|_| "node port must be between 0 and 65535".to_owned())?;
    Ok((host.as_bytes().to_vec(), port))
}

pub(super) fn parse_file_attributes(value: &str) -> Result<(Vec<Vec<u8>>, Vec<u8>), String> {
    let (path, attributes) = value
        .split_once('=')
        .ok_or_else(|| "expected PATH=ATTRIBUTES".to_owned())?;
    let path = path
        .split('/')
        .map(|component| component.as_bytes().to_vec())
        .collect::<Vec<_>>();
    if path.is_empty() || path.iter().any(Vec::is_empty) {
        return Err("file path must contain non-empty components".to_owned());
    }
    Ok((path, attributes.as_bytes().to_vec()))
}
