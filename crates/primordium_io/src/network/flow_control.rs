use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct TokenBucket {
    capacity: f64,
    tokens: Arc<Mutex<f64>>,
    refill_rate: f64,
    last_refill: Arc<Mutex<Instant>>,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: Arc::new(Mutex::new(capacity)),
            refill_rate,
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn try_acquire(&self, amount: f64) -> bool {
        let mut tokens = self.tokens.lock().unwrap();
        let mut last_refill = self.last_refill.lock().unwrap();

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
