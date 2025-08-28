use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum LoadPattern {
    /// Constant load throughout the test
    Constant,
    /// Gradually ramp up from 0 to target over the warmup period
    RampUp,
    /// Spike pattern with bursts of high load
    Spike,
}

pub struct LoadController {
    pattern: LoadPattern,
    target_rps: Option<u32>,
    concurrent_users: u32,
    test_start: Instant,
    warmup_duration: Duration,
}

impl LoadController {
    pub fn new(
        pattern: LoadPattern,
        target_rps: Option<u32>,
        concurrent_users: u32,
        warmup_duration: Duration,
    ) -> Self {
        Self {
            pattern,
            target_rps,
            concurrent_users,
            test_start: Instant::now(),
            warmup_duration,
        }
    }

    /// Calculate the current target RPS based on the load pattern and elapsed time
    pub fn current_target_rps(&self) -> f64 {
        let elapsed = self.test_start.elapsed();
        let base_rps = self.target_rps.unwrap_or(self.concurrent_users * 10) as f64;

        match self.pattern {
            LoadPattern::Constant => base_rps,
            LoadPattern::RampUp => {
                if elapsed < self.warmup_duration {
                    // Gradually increase from 0 to target over warmup period
                    let progress = elapsed.as_secs_f64() / self.warmup_duration.as_secs_f64();
                    base_rps * progress
                } else {
                    base_rps
                }
            }
            LoadPattern::Spike => {
                // Create spikes every 30 seconds
                let cycle_duration = Duration::from_secs(30);
                let cycle_elapsed = elapsed.as_secs_f64() % cycle_duration.as_secs_f64();

                if cycle_elapsed < 5.0 {
                    // 5-second spike at 2x the base rate
                    base_rps * 2.0
                } else {
                    // Normal rate for the rest of the cycle
                    base_rps
                }
            }
        }
    }

    /// Calculate current concurrent user count based on pattern
    #[allow(dead_code)]
    pub fn current_concurrent_users(&self) -> u32 {
        let elapsed = self.test_start.elapsed();

        match self.pattern {
            LoadPattern::Constant => self.concurrent_users,
            LoadPattern::RampUp => {
                if elapsed < self.warmup_duration {
                    let progress = elapsed.as_secs_f64() / self.warmup_duration.as_secs_f64();
                    ((self.concurrent_users as f64) * progress).max(1.0) as u32
                } else {
                    self.concurrent_users
                }
            }
            LoadPattern::Spike => {
                let cycle_duration = Duration::from_secs(30);
                let cycle_elapsed = elapsed.as_secs_f64() % cycle_duration.as_secs_f64();

                if cycle_elapsed < 5.0 {
                    // Spike: double the concurrent users
                    self.concurrent_users * 2
                } else {
                    self.concurrent_users
                }
            }
        }
    }

    /// Calculate delay between requests to achieve target RPS
    pub fn request_delay(&self) -> Option<Duration> {
        self.target_rps.map(|_rps| {
            let current_rps = self.current_target_rps();
            let delay_millis = 1000.0 / current_rps;
            Duration::from_millis(delay_millis as u64)
        })
    }

    /// Get a human-readable description of the current load phase
    pub fn current_phase_description(&self) -> String {
        let elapsed = self.test_start.elapsed();

        match self.pattern {
            LoadPattern::Constant => "Constant load".to_string(),
            LoadPattern::RampUp => {
                if elapsed < self.warmup_duration {
                    let progress =
                        (elapsed.as_secs_f64() / self.warmup_duration.as_secs_f64() * 100.0) as u32;
                    format!("Ramping up ({}%)", progress)
                } else {
                    "Full load".to_string()
                }
            }
            LoadPattern::Spike => {
                let cycle_duration = Duration::from_secs(30);
                let cycle_elapsed = elapsed.as_secs_f64() % cycle_duration.as_secs_f64();

                if cycle_elapsed < 5.0 {
                    format!("Spike phase ({:.1}s remaining)", 5.0 - cycle_elapsed)
                } else {
                    format!("Normal phase ({:.1}s to spike)", 30.0 - cycle_elapsed)
                }
            }
        }
    }
}
