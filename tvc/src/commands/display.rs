//! Shared display helpers for CLI output.

use std::fmt::{self, Display, Formatter};

pub fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

/// Renders the value, or `<unknown>` when it's absent.
pub struct OrUnknown<T>(pub Option<T>);

impl<T: Display> Display for OrUnknown<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => value.fmt(f),
            None => f.write_str("<unknown>"),
        }
    }
}

pub fn format_egress_enabled(enable_egress: bool) -> String {
    format!("Egress Enabled: {}", yes_no(enable_egress))
}

#[cfg(test)]
mod tests {
    use super::format_egress_enabled;

    #[test]
    fn format_egress_enabled_formats_yes_and_no() {
        assert_eq!(format_egress_enabled(true), "Egress Enabled: yes");
        assert_eq!(format_egress_enabled(false), "Egress Enabled: no");
    }
}
