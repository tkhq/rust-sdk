//! Escape data at presentation boundaries, before adding trusted terminal styling.

use serde::Serialize;
use std::fmt::{self, Display, Formatter, Write};

/// Display text without terminal controls or Unicode direction overrides.
/// Newlines and tabs remain available for the CLI's multiline layouts.
pub(crate) struct TerminalText<T>(pub(crate) T);

impl<T: Display> Display for TerminalText<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let text = self.0.to_string();
        let mut start = 0;
        for (index, c) in text.char_indices() {
            if needs_escape(c) && !matches!(c, '\n' | '\t') {
                f.write_str(&text[start..index])?;
                write!(f, "{}", c.escape_default())?;
                start = index + c.len_utf8();
            }
        }
        f.write_str(&text[start..])
    }
}

impl<T: Display> TerminalText<T> {
    /// Keep data fields on one line and flag any escaped control characters.
    pub(crate) fn single_line(&self) -> String {
        let text = self.0.to_string();
        let escaped = TerminalText(&text)
            .to_string()
            .replace('\n', r"\n")
            .replace('\t', r"\t");
        if text == escaped {
            escaped
        } else {
            format!("[warning: escaped control characters] {escaped}")
        }
    }
}

fn needs_escape(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
        )
}

/// JSON already escapes ASCII controls. Escape the remaining controls using
/// JSON syntax too, so decoded values remain identical for machine consumers.
pub(crate) fn terminal_json(value: &impl Serialize) -> serde_json::Result<String> {
    let json = serde_json::to_string(value)?;
    let mut escaped = String::with_capacity(json.len());
    for c in json.chars() {
        if needs_escape(c) {
            // All characters selected by needs_escape are in the BMP.
            write!(escaped, "\\u{:04x}", c as u32).expect("writing to a String cannot fail");
        } else {
            escaped.push(c);
        }
    }
    Ok(escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_are_visible_without_changing_readable_text_or_layout() {
        assert_eq!(
            TerminalText("café 🦀\n\t\x1b[2J\r\x08\x07\0\x7f\u{9b}2J\u{202e}hash\u{2069}")
                .to_string(),
            concat!(
                "café 🦀\n\t",
                r"\u{1b}[2J\r\u{8}\u{7}\u{0}\u{7f}\u{9b}2J\u{202e}hash\u{2069}"
            )
        );
    }

    #[test]
    fn prompt_values_are_single_line() {
        assert_eq!(
            TerminalText("name\n\t\r\x1b[H").single_line(),
            r"[warning: escaped control characters] name\n\t\r\u{1b}[H"
        );
    }

    #[test]
    fn json_escapes_controls_and_round_trips_the_original_data() {
        let value = "\x1b[2J\n\r\u{9b}2J\u{202e}café";
        let json = terminal_json(&value).unwrap();
        assert_eq!(json, r#""\u001b[2J\n\r\u009b2J\u202ecafé""#);
        assert_eq!(serde_json::from_str::<String>(&json).unwrap(), value);
    }
}
