use std::collections::VecDeque;
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct EmailEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: EventType,
    pub email: String,
    pub smtp_server: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EventType {
    Sent,
    Delivered,
    Bounced,
    Opened,
    Clicked,
    Error,
}

pub struct MonitoringSystem {
    events: VecDeque<EmailEvent>,
    max_events: usize,
    start_time: Instant,
}

impl MonitoringSystem {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            max_events: 1000,
            start_time: Instant::now(),
        }
    }
    
    pub fn record_event(&mut self, event: EmailEvent) {
        self.events.push_back(event);
        
        // Garder seulement les derniers événements
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }
    
    pub fn get_recent_events(&self, count: usize) -> Vec<&EmailEvent> {
        self.events.iter().rev().take(count).collect()
    }
    
    pub fn get_success_rate(&self) -> f64 {
        if self.events.is_empty() {
            return 0.0;
        }
        
        let successful = self.events.iter().filter(|e| e.success).count();
        successful as f64 / self.events.len() as f64
    }
    
    pub fn get_uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}