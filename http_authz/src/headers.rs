//! Header binding for http-authz/v1.
//!
//! The token names the bound headers in `tk_request.headers` and carries
//! `tk_header_digest`, one digest per name in the same order. For each name,
//! the verifier takes the received field lines for that name in received
//! order, trims leading and trailing spaces and tabs from each, joins them
//! with `", "`, and hashes the result ([`crate::digest`]). A bound header
//! that is absent from the request is a mismatch. A field value with a byte
//! outside visible ASCII, space, and tab is rejected.
//!
//! hyper delivers lowercase names and keeps repeated fields in order, so the
//! value is rebuilt directly from the received [`HeaderMap`].
//!
//! The test vectors live in `testdata/digests.json`.

use http::HeaderMap;
use sha2::{Digest, Sha256};

use crate::digest::format_digest;

/// Most bound header names.
pub const MAX_HEADER_NAMES: usize = 32;

const FORBIDDEN_HEADER_NAMES: &[&str] = &[
    "authorization",
    "connection",
    "host",
    "keep-alive",
    "proxy-authorization",
    "proxy-connection",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// A header-binding failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    /// More than [`MAX_HEADER_NAMES`] names.
    TooMany,
    /// A name is not a lowercase HTTP token.
    BadName(String),
    /// A name is a pseudo-header, hop-by-hop, `host`, `authorization`, or
    /// `proxy-authorization`.
    Forbidden(String),
    /// A name appears more than once.
    Duplicate(String),
    /// A bound header is absent from the request.
    Absent(String),
    /// A bound value has a byte outside visible ASCII, space, and tab.
    BadValue(String),
}

impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooMany => write!(f, "at most {MAX_HEADER_NAMES} header names"),
            Self::BadName(n) => write!(f, "header name {n:?} is not a lowercase token"),
            Self::Forbidden(n) => write!(f, "header {n:?} must not be bound"),
            Self::Duplicate(n) => write!(f, "header name {n:?} is repeated"),
            Self::Absent(n) => write!(f, "bound header {n:?} is absent from the request"),
            Self::BadValue(n) => write!(
                f,
                "header {n:?} has a byte outside visible ASCII, space, and tab"
            ),
        }
    }
}

pub(crate) fn is_tchar_except_alpha(b: u8) -> bool {
    b.is_ascii_digit()
        || matches!(
            b,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

fn check_header_name(name: &str) -> Result<(), HeaderError> {
    if name.starts_with(':') || FORBIDDEN_HEADER_NAMES.contains(&name) {
        return Err(HeaderError::Forbidden(name.to_owned()));
    }
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || is_tchar_except_alpha(b))
    {
        return Err(HeaderError::BadName(name.to_owned()));
    }
    Ok(())
}

/// Check a list of bound names: at most [`MAX_HEADER_NAMES`] valid names, each once, in any
/// order. An empty list binds nothing.
///
/// # Errors
///
/// Returns the first rule the list breaks.
pub fn check_header_names<S: AsRef<str>>(names: &[S]) -> Result<(), HeaderError> {
    if names.len() > MAX_HEADER_NAMES {
        return Err(HeaderError::TooMany);
    }
    for (i, n) in names.iter().enumerate() {
        let n = n.as_ref();
        check_header_name(n)?;
        if names[..i].iter().any(|m| m.as_ref() == n) {
            return Err(HeaderError::Duplicate(n.to_owned()));
        }
    }
    Ok(())
}

fn check_value(name: &str, bytes: &[u8]) -> Result<(), HeaderError> {
    if bytes
        .iter()
        .all(|b| (0x20..=0x7e).contains(b) || *b == b'\t')
    {
        Ok(())
    } else {
        Err(HeaderError::BadValue(name.to_owned()))
    }
}

/// The normalized value of header `name` in `headers`: every field line in
/// received order, each trimmed of spaces and tabs, joined with `", "`.
///
/// # Errors
///
/// Returns [`HeaderError::Absent`] when the request has no such field, or
/// [`HeaderError::BadValue`] for a byte outside visible ASCII, space, and
/// tab.
pub fn header_value(name: &str, headers: &HeaderMap) -> Result<String, HeaderError> {
    let mut out = String::new();
    join_bound_lines(name, headers, |b| {
        out.extend(b.iter().map(|&c| char::from(c)))
    })?;
    Ok(out)
}

fn join_bound_lines(
    name: &str,
    headers: &HeaderMap,
    mut sink: impl FnMut(&[u8]),
) -> Result<(), HeaderError> {
    if !headers.contains_key(name) {
        return Err(HeaderError::Absent(name.to_owned()));
    }
    for (i, v) in headers.get_all(name).iter().enumerate() {
        check_value(name, v.as_bytes())?;
        if i > 0 {
            sink(b", ");
        }
        sink(v.as_bytes().trim_ascii());
    }
    Ok(())
}

/// The digest of header `name` in `headers` ([`header_value`], hashed).
///
/// # Errors
///
/// As [`header_value`].
pub fn header_digest(name: &str, headers: &HeaderMap) -> Result<String, HeaderError> {
    let mut ctx = Sha256::new();
    join_bound_lines(name, headers, |b| ctx.update(b))?;
    Ok(format_digest(&ctx.finalize()))
}

/// One digest per name in `names`, in order.
///
/// # Errors
///
/// As [`header_value`], for the first name that fails.
pub fn header_digests<S: AsRef<str>>(
    names: &[S],
    headers: &HeaderMap,
) -> Result<Vec<String>, HeaderError> {
    names
        .iter()
        .map(|n| header_digest(n.as_ref(), headers))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::tests::VECTORS;
    use http::{HeaderName, HeaderValue};

    fn header_map(received: &serde_json::Value) -> HeaderMap {
        let mut h = HeaderMap::new();
        for pair in received.as_array().unwrap() {
            let name = HeaderName::from_bytes(pair[0].as_str().unwrap().as_bytes()).unwrap();
            let value = HeaderValue::from_bytes(pair[1].as_str().unwrap().as_bytes()).unwrap();
            h.append(name, value);
        }
        h
    }

    #[test]
    fn header_value_and_digest_match_shared_vectors() {
        let v: serde_json::Value = serde_json::from_str(VECTORS).unwrap();
        let cases = v["headers"].as_array().unwrap();
        assert!(cases.len() >= 8);
        for case in cases {
            let name = case["name"].as_str().unwrap();
            let bound = case["header"].as_str().unwrap();
            let received = header_map(&case["received"]);
            let got = header_value(bound, &received)
                .and_then(|value| Ok((value, header_digest(bound, &received)?)));
            match case.get("value").and_then(|c| c.as_str()) {
                Some(want) => {
                    let (value, digest) = got.unwrap_or_else(|e| panic!("{name}: {e}"));
                    assert_eq!(value, want, "{name}");
                    assert_eq!(digest, case["digest"].as_str().unwrap(), "{name}");
                }
                None => assert!(got.is_err(), "{name} should fail, got {got:?}"),
            }
        }
    }

    #[test]
    fn check_header_names_accepts_and_rejects_shared_name_lists() {
        let v: serde_json::Value = serde_json::from_str(VECTORS).unwrap();
        for case in v["header_names"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            let names: Vec<String> = serde_json::from_value(case["names"].clone()).unwrap();
            assert_eq!(
                check_header_names(&names).is_ok(),
                case["ok"].as_bool().unwrap(),
                "{name}"
            );
        }
    }

    #[test]
    fn rejects_duplicate_oversized_and_forbidden_header_names() {
        assert_eq!(
            check_header_names(&["a", "a"]),
            Err(HeaderError::Duplicate("a".into()))
        );
        let many: Vec<String> = (0..=MAX_HEADER_NAMES)
            .map(|i| format!("x-{i:02}"))
            .collect();
        assert_eq!(check_header_names(&many), Err(HeaderError::TooMany));
        for f in FORBIDDEN_HEADER_NAMES {
            assert_eq!(
                check_header_names(&[*f]),
                Err(HeaderError::Forbidden((*f).to_owned()))
            );
        }
    }
}
