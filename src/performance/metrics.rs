use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub start_time: Instant,
    pub response_times: Vec<Duration>,
    pub error_count: u64,
    pub request_count: u64,
    pub status_codes: HashMap<u16, u64>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_errors: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            response_times: Vec::new(),
            error_count: 0,
            request_count: 0,
            status_codes: HashMap::new(),
            bytes_sent: 0,
            bytes_received: 0,
            connection_errors: 0,
        }
    }

    pub fn record_request(
        &mut self,
        response_time: Duration,
        status_code: u16,
        bytes_sent: u64,
        bytes_received: u64,
        is_error: bool,
    ) {
        self.response_times.push(response_time);
        self.request_count += 1;
        self.bytes_sent += bytes_sent;
        self.bytes_received += bytes_received;

        *self.status_codes.entry(status_code).or_insert(0) += 1;

        if is_error {
            self.error_count += 1;
        }
    }

    pub fn record_connection_error(&mut self) {
        self.connection_errors += 1;
        self.error_count += 1;
    }

    pub fn calculate_results(&self) -> PerformanceResults {
        let total_duration = self.start_time.elapsed();
        let total_requests = self.request_count + self.connection_errors;

        if self.response_times.is_empty() {
            return PerformanceResults {
                total_requests,
                successful_requests: 0,
                failed_requests: self.error_count,
                success_rate: 0.0,
                requests_per_second: 0.0,
                average_response_time: Duration::ZERO,
                min_response_time: Duration::ZERO,
                max_response_time: Duration::ZERO,
                p50_response_time: Duration::ZERO,
                p95_response_time: Duration::ZERO,
                p99_response_time: Duration::ZERO,
                status_code_distribution: self.status_codes.clone(),
                bytes_per_second_sent: 0.0,
                bytes_per_second_received: 0.0,
                connection_errors: self.connection_errors,
                total_duration,
            };
        }

        let mut sorted_times = self.response_times.clone();
        sorted_times.sort();

        let successful_requests = self.request_count - self.error_count + self.connection_errors;
        let success_rate = if total_requests > 0 {
            (total_requests - self.error_count) as f64 / total_requests as f64
        } else {
            0.0
        };

        let avg_response_time = Duration::from_nanos(
            (sorted_times.iter().map(|d| d.as_nanos()).sum::<u128>() / sorted_times.len() as u128)
                as u64,
        );

        let p50_index = sorted_times.len() * 50 / 100;
        let p95_index = sorted_times.len() * 95 / 100;
        let p99_index = sorted_times.len() * 99 / 100;

        PerformanceResults {
            total_requests,
            successful_requests,
            failed_requests: self.error_count,
            success_rate,
            requests_per_second: total_requests as f64 / total_duration.as_secs_f64(),
            average_response_time: avg_response_time,
            min_response_time: sorted_times.first().copied().unwrap_or(Duration::ZERO),
            max_response_time: sorted_times.last().copied().unwrap_or(Duration::ZERO),
            p50_response_time: sorted_times
                .get(p50_index)
                .copied()
                .unwrap_or(Duration::ZERO),
            p95_response_time: sorted_times
                .get(p95_index)
                .copied()
                .unwrap_or(Duration::ZERO),
            p99_response_time: sorted_times
                .get(p99_index)
                .copied()
                .unwrap_or(Duration::ZERO),
            status_code_distribution: self.status_codes.clone(),
            bytes_per_second_sent: self.bytes_sent as f64 / total_duration.as_secs_f64(),
            bytes_per_second_received: self.bytes_received as f64 / total_duration.as_secs_f64(),
            connection_errors: self.connection_errors,
            total_duration,
        }
    }

    #[allow(dead_code)]
    pub fn merge(&mut self, other: &PerformanceMetrics) {
        self.response_times.extend(other.response_times.iter());
        self.error_count += other.error_count;
        self.request_count += other.request_count;
        self.bytes_sent += other.bytes_sent;
        self.bytes_received += other.bytes_received;
        self.connection_errors += other.connection_errors;

        for (status, count) in &other.status_codes {
            *self.status_codes.entry(*status).or_insert(0) += count;
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceResults {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub requests_per_second: f64,

    #[serde(with = "duration_serde")]
    pub average_response_time: Duration,
    #[serde(with = "duration_serde")]
    pub min_response_time: Duration,
    #[serde(with = "duration_serde")]
    pub max_response_time: Duration,
    #[serde(with = "duration_serde")]
    pub p50_response_time: Duration,
    #[serde(with = "duration_serde")]
    pub p95_response_time: Duration,
    #[serde(with = "duration_serde")]
    pub p99_response_time: Duration,

    pub status_code_distribution: HashMap<u16, u64>,
    pub bytes_per_second_sent: f64,
    pub bytes_per_second_received: f64,
    pub connection_errors: u64,

    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
}

impl PerformanceResults {
    pub fn save_report(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

// Helper module for serializing Duration as milliseconds
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}
