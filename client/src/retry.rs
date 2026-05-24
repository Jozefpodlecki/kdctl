use std::time::Duration;
use std::thread::sleep;

use rand::RngExt;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
    pub max_retries: Option<usize>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
            max_retries: Some(10),
        }
    }
}

pub struct RetryPolicy {
    config: RetryConfig,
    attempt: usize,
    current_delay: Duration,
}

impl RetryPolicy {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            current_delay: config.initial_delay,
            attempt: 0,
            config,
        }
    }

    pub fn should_retry(&mut self) -> bool {
        self.attempt += 1;

        if let Some(max) = self.config.max_retries {
            if self.attempt > max {
                return false;
            }
        }

        let next_delay = self.current_delay.as_secs_f64() * self.config.backoff_factor;
        self.current_delay = Duration::from_secs_f64(next_delay.min(self.config.max_delay.as_secs_f64()));

        true
    }

    pub fn wait(&self) {
        sleep(self.current_delay);
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
        self.current_delay = self.config.initial_delay;
    }

    pub fn current_delay(&self) -> Duration {
        self.current_delay
    }

    pub fn attempt(&self) -> usize {
        self.attempt
    }
}

pub fn exponential_backoff_with_jitter(attempt: u32, base_delay: Duration, max_delay: Duration) -> Duration {
    let exp = 2_u32.pow(attempt);
    let delay = base_delay.as_secs() * exp as u64;
    let delay = delay.min(max_delay.as_secs());

    let jitter = rand::rng().random_range(0..=delay);
    Duration::from_secs(jitter)
}