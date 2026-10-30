use std::fmt;

/// A token verification failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The token is not a well-formed compact JWS, or its header or payload
    /// does not parse.
    Malformed(&'static str),
    /// `alg` is not `ES256`.
    UnsupportedAlgorithm,
    /// `typ` is not the media type the caller expects.
    WrongType,
    /// The header does not select a trusted key.
    UnknownKey,
    /// The signature does not verify.
    BadSignature,
    /// A claim is missing, has the wrong type, or has a bad value.
    BadClaim(&'static str),
    /// `exp - iat` exceeds the allowed maximum.
    LifetimeTooLong,
    /// `iat` is in the future beyond the allowed skew.
    NotYetValid,
    /// The token has expired.
    Expired,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(d) => write!(f, "malformed token: {d}"),
            Self::UnsupportedAlgorithm => f.write_str("unsupported alg"),
            Self::WrongType => f.write_str("wrong typ"),
            Self::UnknownKey => f.write_str("unknown key"),
            Self::BadSignature => f.write_str("bad signature"),
            Self::BadClaim(d) => write!(f, "bad claim: {d}"),
            Self::LifetimeTooLong => f.write_str("lifetime too long"),
            Self::NotYetValid => f.write_str("not yet valid"),
            Self::Expired => f.write_str("expired"),
        }
    }
}

impl std::error::Error for Error {}
