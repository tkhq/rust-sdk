//! Turnkey identifiers as they appear in token claims.

/// Whether `s` is a UUID in canonical lowercase form, `8-4-4-4-12` hex digits.
#[must_use]
pub fn is_uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| match i {
            8 | 13 | 18 | 23 => b == b'-',
            _ => b.is_ascii_digit() || (b'a'..=b'f').contains(&b),
        })
}

/// The UUID in `urn`, when `urn` is `prefix` followed by a UUID.
#[must_use]
pub fn urn_uuid<'a>(urn: &'a str, prefix: &str) -> Option<&'a str> {
    urn.strip_prefix(prefix).filter(|id| is_uuid(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "4f0a3a8e-9d2b-4c61-8f57-2b1c6d0e7a11";

    #[test]
    fn accepts_a_canonical_lowercase_uuid() {
        assert!(is_uuid(ID));
        assert!(is_uuid("00000000-0000-0000-0000-000000000000"));
    }

    #[test]
    fn rejects_anything_else() {
        for s in [
            "",
            "parent_org_1",
            &ID.to_uppercase(),
            &ID.replace('-', ""),
            &format!("{ID}0"),
            &ID[1..],
            "4f0a3a8e-9d2b-4c61-8f57-2b1c6d0e7a1g",
            "4f0a3a8e_9d2b-4c61-8f57-2b1c6d0e7a11",
            "4f0a3a8e-9d2b-4c61-8f57-2b1c6d0e7a1\u{e9}",
        ] {
            assert!(!is_uuid(s), "{s}");
        }
    }

    #[test]
    fn urn_uuid_needs_the_prefix_and_a_uuid() {
        let prefix = "urn:turnkey:org:";
        assert_eq!(urn_uuid(&format!("{prefix}{ID}"), prefix), Some(ID));
        assert_eq!(urn_uuid(ID, prefix), None);
        assert_eq!(urn_uuid(&format!("{prefix}org_1"), prefix), None);
        assert_eq!(urn_uuid(&format!("urn:turnkey:user:{ID}"), prefix), None);
    }
}
