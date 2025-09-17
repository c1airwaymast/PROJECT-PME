use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpConfig {
    /// Nom unique du serveur SMTP
    pub name: String,
    
    /// Serveur activé ou non
    pub enabled: bool,
    
    /// Hôte SMTP
    pub smtp_host: String,
    
    /// Port SMTP
    pub smtp_port: u16,
    
    /// Email d'expédition
    pub email: String,
    
    /// Nom d'utilisateur (souvent identique à email)
    pub username: String,
    
    /// Mot de passe ou clé API
    pub password: String,
    
    /// Utilise une API au lieu de SMTP classique
    pub smtp_api: bool,
    
    /// Endpoint API si smtp_api = true
    pub api_endpoint: Option<String>,
    
    /// Type de chiffrement (tls, starttls, none)
    pub encryption: String,
    
    /// Limite d'emails par jour pour ce SMTP
    pub daily_limit: u32,
    
    /// Limite d'emails par heure
    pub hourly_limit: u32,
    
    /// Délai minimum entre envois (millisecondes)
    pub min_delay_ms: u64,
    
    /// Configuration DKIM
    pub dkim: Option<DkimConfig>,
    
    /// Headers personnalisés pour ce SMTP
    pub custom_headers: HashMap<String, String>,
    
    /// Priorité (1 = haute, 10 = basse)
    pub priority: u8,
    
    /// Poids pour la répartition de charge
    pub weight: u32,
    
    /// Domaines préférés pour ce SMTP
    pub preferred_domains: Vec<String>,
    
    /// Configuration de retry spécifique
    pub retry_config: RetryConfig,
    
    /// Monitoring spécifique
    pub monitoring: SmtpMonitoring,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DkimConfig {
    /// Domaine DKIM
    pub domain: String,
    
    /// Sélecteur DKIM
    pub selector: String,
    
    /// Chemin vers la clé privée
    pub private_key_path: String,
    
    /// Algorithme (rsa-sha256, rsa-sha1)
    pub algorithm: String,
    
    /// Canonicalisation (relaxed/relaxed, simple/simple)
    pub canonicalization: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    /// Nombre maximum de tentatives
    pub max_attempts: u32,
    
    /// Délai initial (secondes)
    pub initial_delay: u64,
    
    /// Multiplicateur pour backoff exponentiel
    pub backoff_multiplier: f64,
    
    /// Délai maximum (secondes)
    pub max_delay: u64,
    
    /// Types d'erreurs à retry
    pub retry_on_errors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpMonitoring {
    /// Activer le monitoring pour ce SMTP
    pub enabled: bool,
    
    /// Seuil d'alerte pour le taux d'échec (%)
    pub failure_rate_threshold: f32,
    
    /// Seuil d'alerte pour le temps de réponse (ms)
    pub response_time_threshold: u64,
    
    /// Activer les alertes
    pub alerts_enabled: bool,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            enabled: false,
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            email: "noreply@example.com".to_string(),
            username: "noreply@example.com".to_string(),
            password: "password".to_string(),
            smtp_api: false,
            api_endpoint: None,
            encryption: "starttls".to_string(),
            daily_limit: 1000,
            hourly_limit: 100,
            min_delay_ms: 100,
            dkim: None,
            custom_headers: HashMap::new(),
            priority: 5,
            weight: 100,
            preferred_domains: vec![],
            retry_config: RetryConfig {
                max_attempts: 3,
                initial_delay: 5,
                backoff_multiplier: 2.0,
                max_delay: 300,
                retry_on_errors: vec![
                    "timeout".to_string(),
                    "connection_refused".to_string(),
                    "temporary_failure".to_string(),
                ],
            },
            monitoring: SmtpMonitoring {
                enabled: true,
                failure_rate_threshold: 10.0,
                response_time_threshold: 5000,
                alerts_enabled: false,
            },
        }
    }
}

impl SmtpConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Le nom SMTP ne peut pas être vide"));
        }
        
        if self.smtp_host.is_empty() {
            return Err(anyhow::anyhow!("L'hôte SMTP ne peut pas être vide"));
        }
        
        if self.email.is_empty() {
            return Err(anyhow::anyhow!("L'email ne peut pas être vide"));
        }
        
        if !self.email.contains('@') {
            return Err(anyhow::anyhow!("Format d'email invalide"));
        }
        
        if self.smtp_port == 0 {
            return Err(anyhow::anyhow!("Port SMTP invalide"));
        }
        
        if self.daily_limit == 0 {
            return Err(anyhow::anyhow!("La limite quotidienne doit être > 0"));
        }
        
        if self.hourly_limit == 0 {
            return Err(anyhow::anyhow!("La limite horaire doit être > 0"));
        }
        
        if self.priority == 0 || self.priority > 10 {
            return Err(anyhow::anyhow!("La priorité doit être entre 1 et 10"));
        }
        
        Ok(())
    }
    
    pub fn is_preferred_for_domain(&self, domain: &str) -> bool {
        if self.preferred_domains.is_empty() {
            return true; // Accepte tous les domaines si aucune préférence
        }
        
        self.preferred_domains.iter().any(|d| {
            d == "*" || d == domain || domain.ends_with(&format!(".{}", d))
        })
    }
    
    pub fn get_connection_string(&self) -> String {
        format!("{}://{}:{}@{}:{}", 
                if self.encryption == "tls" { "smtps" } else { "smtp" },
                self.username,
                "***", // Masquer le mot de passe
                self.smtp_host,
                self.smtp_port)
    }
}