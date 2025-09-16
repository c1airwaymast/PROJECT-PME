use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    /// Chiffrement activé
    pub encryption_enabled: bool,
    
    /// DKIM activé
    pub dkim_enabled: bool,
    pub dkim_domain: String,
    pub dkim_selector: String,
    pub dkim_private_key_path: String,
    
    /// Rate limiting global
    pub global_rate_limit: u32,
    
    /// Protection anti-spam
    pub anti_spam_headers: bool,
    pub reputation_protection: bool,
    
    /// Monitoring sécurité
    pub security_monitoring: bool,
    pub alert_on_high_bounce_rate: bool,
    pub max_bounce_rate_percent: f32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            dkim_enabled: false,
            dkim_domain: "example.com".to_string(),
            dkim_selector: "default".to_string(),
            dkim_private_key_path: "keys/dkim.key".to_string(),
            global_rate_limit: 10,
            anti_spam_headers: true,
            reputation_protection: true,
            security_monitoring: true,
            alert_on_high_bounce_rate: true,
            max_bounce_rate_percent: 5.0,
        }
    }
}