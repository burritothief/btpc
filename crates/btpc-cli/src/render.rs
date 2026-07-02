use crate::output::display_width;
use std::fmt::Write as _;

// Spec: CLI-OUTPUT-001
pub(crate) fn key_values(rows: &[(&str, String)], pretty: bool, width: usize) -> String {
    if !pretty {
        return rows.iter().fold(String::new(), |mut output, (key, value)| {
            writeln!(output, "{key}: {value}").expect("writing to String cannot fail");
            output
        });
    }
    let key_width = rows
        .iter()
        .map(|(key, _)| display_width(key))
        .max()
        .unwrap_or(0);
    rows.iter().fold(String::new(), |mut output, (key, value)| {
        let padding = " ".repeat(key_width.saturating_sub(display_width(key)));
        let prefix = format!("• {key}{padding}  ");
        let available = width.saturating_sub(display_width(&prefix));
        writeln!(output, "{prefix}{}", truncate(value, available))
            .expect("writing to String cannot fail");
        output
    })
}

fn truncate(value: &str, width: usize) -> String {
    if display_width(value) <= width {
        return value.to_owned();
    }
    if width <= 1 {
        return "…".to_owned();
    }
    let mut result = String::new();
    for character in value.chars() {
        if display_width(&result) + display_width(&character.to_string()) + 1 > width {
            break;
        }
        result.push(character);
    }
    result.push('…');
    result
}

#[cfg(test)]
mod tests {
    use super::key_values;

    #[test]
    fn compact_and_pretty_rendering_are_deterministic_at_multiple_widths() {
        let rows = [
            ("mode", "v2".to_owned()),
            ("long field", "abcdefghijk".to_owned()),
        ];
        assert_eq!(
            key_values(&rows, false, 80),
            "mode: v2\nlong field: abcdefghijk\n"
        );
        assert_eq!(
            key_values(&rows, true, 80),
            "• mode        v2\n• long field  abcdefghijk\n"
        );
        assert_eq!(
            key_values(&rows, true, 18),
            "• mode        v2\n• long field  abc…\n"
        );
    }
}
