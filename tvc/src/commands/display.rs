//! Shared display helpers for CLI output.

pub fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

pub fn egress_enabled_line(enable_egress: bool) -> String {
    format!("Egress Enabled: {}", yes_no(enable_egress))
}

#[cfg(test)]
mod tests {
    use super::egress_enabled_line;

    #[test]
    fn egress_enabled_line_formats_yes_and_no() {
        assert_eq!(egress_enabled_line(true), "Egress Enabled: yes");
        assert_eq!(egress_enabled_line(false), "Egress Enabled: no");
    }
}
