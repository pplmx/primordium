use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Thread-safe token bucket implementation for rate-limiting network operations.
#[derive(Clone, Debug)]
pub struct TokenBucket {
    capacity: f64,
    tokens: Arc<Mutex<f64>>,
    refill_rate: f64,
    last_refill: Arc<Mutex<Instant>>,
}

impl TokenBucket {
    /// Creates a new token bucket with specified capacity and refill rate (tokens per second).
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: Arc::new(Mutex::new(capacity)),
            refill_rate,
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Attempts to acquire the specified number of tokens.
    ///
    /// Returns `true` if tokens were available, `false` otherwise.
    /// Automatically refills tokens based on elapsed time since last call.
    pub fn try_acquire(&self, amount: f64) -> bool {
        let mut tokens = self.tokens.lock().expect("Token bucket mutex poisoned");
        let mut last_refill = self
            .last_refill
            .lock()
            .expect("Token bucket mutex poisoned");

        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;

        if new_tokens > 0.0 {
            *tokens = (*tokens + new_tokens).min(self.capacity);
            *last_refill = now;
        }

        if *tokens >= amount {
            *tokens -= amount;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_token_bucket_rate_limiting() {
        let bucket = TokenBucket::new(5.0, 10.0); // 5 tokens max, 10 per sec

        // Should consume initial capacity
        assert!(bucket.try_acquire(5.0));
        assert!(!bucket.try_acquire(1.0)); // Empty

        // Wait for refill (0.1s -> 1 token)
        thread::sleep(Duration::from_millis(110));
        assert!(bucket.try_acquire(1.0));
    }
}
