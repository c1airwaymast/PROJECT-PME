use std::time::Instant;

pub struct PerformanceMonitor {
    start_time: Instant,
    emails_processed: u64,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            emails_processed: 0,
        }
    }
    
    pub fn record_email_sent(&mut self) {
        self.emails_processed += 1;
    }
    
    pub fn get_rate_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.emails_processed as f64 / elapsed
        } else {
            0.0
        }
    }
    
    pub fn get_uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}