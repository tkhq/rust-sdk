//! Generic `iat` and `exp` checks.

use crate::Error;

/// Time bounds for [`check_times`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct TimeRules {
    /// Allowed clock skew in seconds for `iat`.
    pub clock_skew_secs: u64,
    /// Longest accepted `exp - iat` in seconds.
    pub max_lifetime_secs: u64,
}

impl TimeRules {
    /// Time bounds with this skew and maximum lifetime.
    #[must_use]
    pub const fn new(clock_skew_secs: u64, max_lifetime_secs: u64) -> Self {
        Self {
            clock_skew_secs,
            max_lifetime_secs,
        }
    }
}

/// Check `iat` and `exp` (NumericDate seconds) against `now`.
///
/// The checks run in this order: `exp > iat`, `exp - iat` at most
/// `max_lifetime_secs`, `iat <= now + clock_skew_secs`, and `now < exp`.
///
/// # Errors
///
/// Returns [`Error::BadClaim`], [`Error::LifetimeTooLong`],
/// [`Error::NotYetValid`], or [`Error::Expired`] for the first failed check.
pub fn check_times(iat: u64, exp: u64, now: u64, rules: &TimeRules) -> Result<(), Error> {
    if exp <= iat {
        return Err(Error::BadClaim("exp must be after iat"));
    }
    if exp - iat > rules.max_lifetime_secs {
        return Err(Error::LifetimeTooLong);
    }
    if iat > now.saturating_add(rules.clock_skew_secs) {
        return Err(Error::NotYetValid);
    }
    if now >= exp {
        return Err(Error::Expired);
    }
    Ok(())
}

/// Current Unix time in seconds. Returns u64::MAX if the clock is before 1970,
/// so the expiry check (now >= exp) fails closed.
#[must_use]
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_run_in_order_with_inclusive_bounds() {
        let r = TimeRules::new(5, 300);
        let t = 1_800_000_000;
        assert_eq!(check_times(t, t + 300, t + 299, &r), Ok(()));
        assert_eq!(check_times(t, t + 300, t + 300, &r), Err(Error::Expired));
        assert_eq!(check_times(t, t + 301, t, &r), Err(Error::LifetimeTooLong));
        assert_eq!(check_times(t + 5, t + 30, t, &r), Ok(()));
        assert_eq!(check_times(t + 6, t + 30, t, &r), Err(Error::NotYetValid));
        assert_eq!(
            check_times(t, t, t, &r),
            Err(Error::BadClaim("exp must be after iat"))
        );
        assert_eq!(check_times(0, 300, u64::MAX, &r), Err(Error::Expired));
    }
}
