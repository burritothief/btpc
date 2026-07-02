use btpc_core::magnet::MagnetOptions;

use crate::output::{REDACTED_URL, display_bytes};

pub(super) fn iec_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut unit = 0;
    let mut divisor = 1_u64;
    while unit + 1 < UNITS.len() && bytes >= divisor.saturating_mul(1024) {
        unit += 1;
        divisor = divisor.saturating_mul(1024);
    }
    let tenths = (u128::from(bytes) * 10 + u128::from(divisor / 2)) / u128::from(divisor);
    format!("{}.{:01} {}", tenths / 10, tenths % 10, UNITS[unit])
}

pub(super) fn display_url(bytes: &[u8]) -> String {
    let value = display_bytes(bytes);
    if value.contains('@') || value.contains("passkey=") {
        REDACTED_URL.to_owned()
    } else {
        value
    }
}

pub(super) fn inspect_magnet(torrent: &btpc_core::Metainfo) -> String {
    let include_trackers = torrent
        .trackers()
        .iter()
        .flatten()
        .all(|url| !url_is_secret(url));
    let include_web_seeds = torrent.web_seeds().iter().all(|url| !url_is_secret(url));
    torrent.magnet(
        &MagnetOptions::builder()
            .trackers(include_trackers)
            .web_seeds(include_web_seeds)
            .build(),
    )
}

fn url_is_secret(bytes: &[u8]) -> bool {
    let value = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    value.contains('@') || value.contains("passkey=")
}

pub(super) fn format_creation_date(timestamp: i64) -> String {
    use chrono::{Local, TimeZone as _};
    Local.timestamp_opt(timestamp, 0).single().map_or_else(
        || timestamp.to_string(),
        |date| date.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::iec_size;

    #[test]
    fn iec_sizes_have_stable_boundaries_and_rounding() {
        assert_eq!(iec_size(0), "0 B");
        assert_eq!(iec_size(1023), "1023 B");
        assert_eq!(iec_size(1024), "1.0 KiB");
        assert_eq!(iec_size(1536), "1.5 KiB");
        assert_eq!(iec_size(1024 * 1024), "1.0 MiB");
        assert_eq!(iec_size(3_989_078_016), "3.7 GiB");
    }
}
