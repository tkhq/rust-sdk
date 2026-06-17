//! Shared display helpers for CLI output.

pub fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
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
