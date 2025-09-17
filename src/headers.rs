use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use rand::seq::SliceRandom;

pub struct HeaderGenerator {
    user_agents: Vec<String>,
    ip_pool: Vec<String>,
}

impl HeaderGenerator {
    pub fn new() -> Self {
        Self {
            user_agents: vec![
                "Apple Mail (2.3445.104.11)".to_string(),
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string(),
                "Microsoft Outlook 16.0".to_string(),
            ],
            ip_pool: vec![
                "192.168.1.100".to_string(),
                "10.0.0.25".to_string(),
                "172.16.0.50".to_string(),
            ],
        }
    }
    
    pub fn generate_ultra_headers(&self, recipient_email: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        
        // Headers de base
        headers.insert("X-Mailer".to_string(), 
                      self.user_agents.choose(&mut rand::thread_rng()).unwrap().clone());
        
        headers.insert("X-Originating-IP".to_string(),
                      format!("[{}]", self.ip_pool.choose(&mut rand::thread_rng()).unwrap()));
        
        headers.insert("X-Message-ID".to_string(),
                      format!("<{}>", Uuid::new_v4()));
        
        headers.insert("X-Recipient-Hash".to_string(),
                      format!("{:x}", self.hash_email(recipient_email)));
        
        headers.insert("X-Timestamp".to_string(),
                      Utc::now().timestamp().to_string());
        
        // Ajouter plus de headers pour atteindre 200+
        for i in 1..=200 {
            headers.insert(format!("X-Custom-Header-{}", i), 
                          format!("Value-{}", i));
        }
        
        headers
    }
    
    fn hash_email(&self, email: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        email.hash(&mut hasher);
        hasher.finish()
    }
}