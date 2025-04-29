//! Turnkey Client to interact with the Turnkey API
//! See <https://docs.turnkey.com>
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Initial delay before the first retry
    pub initial_delay: Duration,
    /// Multiplier for `initial_delay`, yielding the next delay
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
            max_retries: 5,
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

    /// Computes the delay to wait after N attempts have been made.
    /// `attempt_count` starts at 1 because it never makes sense to wait before the first try.
    pub fn compute_delay(&self, attempt_count: usize) -> Duration {
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
        let mut delay = self.initial_delay.mul_f64(factor);

        if delay > self.max_delay {
            delay = self.max_delay;
        }

        delay
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
    #[should_panic(expected = "attempt_count (6) is greater than max_retries (5)")]
    fn test_compute_delay_panics_after_max_attempts() {
        RetryConfig::default().compute_delay(6);
    }

    #[test]
    fn test_default_retry() {
        let config = RetryConfig::default();
        assert_eq!(config.compute_delay(1), Duration::from_millis(500));
        assert_eq!(config.compute_delay(2), Duration::from_millis(1000));
        assert_eq!(config.compute_delay(3), Duration::from_secs(2));
        assert_eq!(config.compute_delay(4), Duration::from_secs(4));
        assert_eq!(config.compute_delay(5), Duration::from_secs(5));
    }

    #[test]
    fn test_long_retry_count() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(500),
            multiplier: 2.0,
            max_delay: Duration::from_secs(10),
            max_retries: 8,
        };

        // Same as default config, except we allow more retries to see "max_delay" kick in.
        assert_eq!(config.compute_delay(1), Duration::from_millis(500));
        assert_eq!(config.compute_delay(2), Duration::from_millis(1000));
        assert_eq!(config.compute_delay(3), Duration::from_secs(2));
        assert_eq!(config.compute_delay(4), Duration::from_secs(4));
        assert_eq!(config.compute_delay(5), Duration::from_secs(8));
        assert_eq!(config.compute_delay(6), Duration::from_secs(10));
        assert_eq!(config.compute_delay(7), Duration::from_secs(10));
        assert_eq!(config.compute_delay(8), Duration::from_secs(10));
    }
}
