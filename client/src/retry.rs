//! Turnkey Client to interact with the Turnkey API
//! See <https://docs.turnkey.com>
use rand::Rng;
use std::{ops::RangeInclusive, time::Duration};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// First backoff, before capping and jitter.
    pub initial_delay: Duration,
    /// Scales the backoff after each attempt.
    pub multiplier: f64,
    /// Maximum delay between retries to cap the time between retries.
    pub max_delay: Duration,
    /// Maximum number of retries.
    pub max_retries: usize,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            initial_delay: Duration::from_millis(500),
            multiplier: 2.0,
            max_delay: Duration::from_secs(5),
            max_retries: 6,
        }
    }
}

impl RetryConfig {
    /// Returns a `RetryConfig` which doesn't allow any retries
    /// Use this if you do not want the TurnkeyClient to retry on your behalf.
    /// If you need a retry configuration, look at `::default()`
    pub fn none() -> Self {
        RetryConfig {
            initial_delay: Duration::from_millis(0),
            multiplier: 0.0,
            max_delay: Duration::from_secs(0),
            max_retries: 0,
        }
    }

    /// Returns the delay to wait after `attempt_count` attempts. It starts at 1.
    ///
    /// The backoff is `initial_delay * multiplier^(attempt_count - 1)`, capped at `max_delay`.
    /// The delay is a random value in `[backoff / 2, backoff]` ("equal jitter"), so clients
    /// do not retry in lockstep and the result never exceeds `max_delay`.
    /// Extreme attempt counts or multipliers are not supported.
    ///
    /// # Panics
    ///
    /// Panics if `attempt_count` is 0 or greater than `max_retries`.
    pub fn compute_delay(&self, attempt_count: usize) -> Duration {
        self.compute_delay_with(attempt_count, |range| rand::thread_rng().gen_range(range))
    }

    /// Like [`Self::compute_delay`], with the choice from the jitter range made by `pick`.
    fn compute_delay_with(
        &self,
        attempt_count: usize,
        pick: impl FnOnce(RangeInclusive<Duration>) -> Duration,
    ) -> Duration {
        if attempt_count == 0 {
            panic!("attempt_count cannot be 0")
        }

        if attempt_count > self.max_retries {
            panic!(
                "attempt_count ({}) is greater than max_retries ({})",
                attempt_count, self.max_retries
            );
        }

        // Compute the factor. If our multiplier is m:
        // - we should wait "delay" ( = delay*1 = delay*(m^0)) after the first attempt
        // - we should wait "delay*m ( = delay*(m^1)) after the second attempt
        // - we should wait "delay*m*m ( = delay*(m^2)) after the third attempt
        let factor = self.multiplier.powi((attempt_count - 1) as i32);
        let backoff = self.initial_delay.mul_f64(factor).min(self.max_delay);

        pick(backoff / 2..=backoff)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "attempt_count (1) is greater than max_retries (0)")]
    fn test_no_retry() {
        let config = RetryConfig::none();
        assert_eq!(config.compute_delay(1), Duration::from_secs(0));
    }

    #[test]
    #[should_panic(expected = "attempt_count cannot be 0")]
    fn test_compute_delay_panics_with_0() {
        RetryConfig::default().compute_delay(0);
    }

    #[test]
    #[should_panic(expected = "attempt_count (7) is greater than max_retries (6)")]
    fn test_compute_delay_panics_after_max_attempts() {
        RetryConfig::default().compute_delay(7);
    }

    fn default_config_with_retries(max_retries: usize) -> RetryConfig {
        RetryConfig {
            max_delay: Duration::from_secs(10),
            max_retries,
            ..RetryConfig::default()
        }
    }

    fn lowest(range: RangeInclusive<Duration>) -> Duration {
        *range.start()
    }

    fn highest(range: RangeInclusive<Duration>) -> Duration {
        *range.end()
    }

    #[test]
    fn test_lowest_pick_is_half_the_backoff() {
        let config = RetryConfig::default();
        assert_eq!(
            config.compute_delay_with(1, lowest),
            Duration::from_millis(250)
        );
        assert_eq!(
            config.compute_delay_with(2, lowest),
            Duration::from_millis(500)
        );
        assert_eq!(config.compute_delay_with(3, lowest), Duration::from_secs(1));
        assert_eq!(config.compute_delay_with(4, lowest), Duration::from_secs(2));
    }

    #[test]
    fn test_highest_pick_is_full_exponential_backoff() {
        let config = default_config_with_retries(8);
        assert_eq!(
            config.compute_delay_with(1, highest),
            Duration::from_millis(500)
        );
        assert_eq!(
            config.compute_delay_with(2, highest),
            Duration::from_millis(1000)
        );
        assert_eq!(
            config.compute_delay_with(3, highest),
            Duration::from_secs(2)
        );
        assert_eq!(
            config.compute_delay_with(4, highest),
            Duration::from_secs(4)
        );
        assert_eq!(
            config.compute_delay_with(5, highest),
            Duration::from_secs(8)
        );
    }

    #[test]
    fn test_max_delay_caps_jittered_delay() {
        let config = default_config_with_retries(8);
        assert_eq!(
            config.compute_delay_with(6, highest),
            Duration::from_secs(10)
        );
        assert_eq!(
            config.compute_delay_with(8, highest),
            Duration::from_secs(10)
        );
        assert_eq!(config.compute_delay_with(8, lowest), Duration::from_secs(5));
    }

    #[test]
    fn test_cap_is_exact_when_not_representable_as_f64() {
        // 2^24 s + 2 ns is not exactly representable as f64 seconds.
        let max_delay = Duration::new(1 << 24, 2);
        let config = RetryConfig {
            initial_delay: Duration::from_secs(1 << 25),
            multiplier: 1.0,
            max_delay,
            max_retries: 1,
        };
        assert_eq!(config.compute_delay_with(1, highest), max_delay);
        assert_eq!(config.compute_delay_with(1, lowest), max_delay / 2);
    }

    #[test]
    fn test_default_max_retries() {
        assert_eq!(RetryConfig::default().max_retries, 6);
    }

    #[test]
    fn test_compute_delay_stays_within_bounds() {
        let config = default_config_with_retries(8);
        for attempt in 1..=8 {
            let backoff = config.compute_delay_with(attempt, highest);
            assert!(backoff <= config.max_delay);
            for _ in 0..100 {
                let delay = config.compute_delay(attempt);
                assert!(delay >= backoff / 2, "{delay:?} below lower bound");
                assert!(delay <= backoff, "{delay:?} above upper bound");
            }
        }
    }
}
