use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

pub mod smtp;
pub mod modes;
pub mod security;

pub use smtp::SmtpConfig;
pub use modes::{ModeConfig, RotationConfig};
pub use security::SecurityConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UltraConfig {
    /// Configuration générale
    pub general: GeneralConfig,
    
    /// Configuration des serveurs SMTP
    pub smtp_servers: Vec<SmtpConfig>,
    
    /// Configuration des modes d'envoi
    pub sending_modes: HashMap<String, ModeConfig>,
    
    /// Configuration des rotations
    pub rotation: RotationConfig,
    
    /// Configuration de sécurité
    pub security: SecurityConfig,
    
    /// Configuration des fichiers
    pub files: FilesConfig,
    
    /// Configuration des variables dynamiques
    pub variables: VariablesConfig,
    
    /// Configuration des X-Headers
    pub headers: HeadersConfig,
    
    /// Configuration du monitoring
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    /// Nom de l'application
    pub app_name: String,
    
    /// Version
    pub version: String,
    
    /// Environnement (dev, staging, prod)
    pub environment: String,
    
    /// Timezone
    pub timezone: String,
    
    /// Langue par défaut
    pub default_language: String,
    
    /// Limite globale d'emails par jour
    pub daily_email_limit: u32,
    
    /// Nombre de threads par défaut
    pub default_threads: usize,
    
    /// Timeout par défaut (secondes)
    pub default_timeout: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilesConfig {
    /// Répertoire des templates
    pub templates_dir: String,
    
    /// Répertoire des logs
    pub logs_dir: String,
    
    /// Répertoire des données
    pub data_dir: String,
    
    /// Répertoire des backups
    pub backups_dir: String,
    
    /// Template HTML par défaut
    pub default_template: String,
    
    /// Fichier de base de données
    pub database_file: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VariablesConfig {
    /// Variables personnalisées globales
    pub global_variables: HashMap<String, String>,
    
    /// Générateurs de variables dynamiques
    pub generators: HashMap<String, VariableGenerator>,
    
    /// Format des dates
    pub date_format: String,
    
    /// Format des heures
    pub time_format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CCEmailConfig {
    pub email: String,
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VariableGenerator {
    pub generator_type: String,
    pub options: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeadersConfig {
    /// Nombre de X-Headers à générer
    pub headers_count: usize,
    
    /// Types de headers à inclure
    pub header_types: Vec<String>,
    
    /// Headers personnalisés statiques
    pub custom_headers: HashMap<String, String>,
    
    /// Rotation des User-Agent
    pub rotate_user_agents: bool,
    
    /// Pool d'IPs pour X-Originating-IP
    pub ip_pool: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    /// Activer le monitoring
    pub enabled: bool,
    
    /// Port pour les métriques Prometheus
    pub metrics_port: u16,
    
    /// Intervalle de collecte (secondes)
    pub collection_interval: u64,
    
    /// Rétention des logs (jours)
    pub log_retention_days: u32,
    
    /// Alertes par email
    pub email_alerts: bool,
    
    /// Webhook pour notifications
    pub webhook_url: Option<String>,
}

impl UltraConfig {
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let config: UltraConfig = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    pub async fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        fs::write(path, content).await?;
        Ok(())
    }
    
    pub fn validate(&self) -> Result<()> {
        // Valider que au moins un SMTP est configuré
        if self.smtp_servers.is_empty() {
            return Err(anyhow::anyhow!("Au moins un serveur SMTP doit être configuré"));
        }
        
        // Valider que au moins un SMTP est activé
        if !self.smtp_servers.iter().any(|s| s.enabled) {
            return Err(anyhow::anyhow!("Au moins un serveur SMTP doit être activé"));
        }
        
        // Valider les modes d'envoi
        if self.sending_modes.is_empty() {
            return Err(anyhow::anyhow!("Au moins un mode d'envoi doit être configuré"));
        }
        
        Ok(())
    }
    
    pub fn get_active_smtp_servers(&self) -> Vec<&SmtpConfig> {
        self.smtp_servers.iter().filter(|s| s.enabled).collect()
    }
    
    pub fn get_sending_mode(&self, mode_name: &str) -> Option<&ModeConfig> {
        self.sending_modes.get(mode_name)
    }
}

impl Default for UltraConfig {
    fn default() -> Self {
        let mut sending_modes = HashMap::new();
        
        // Mode naturel (par défaut)
        sending_modes.insert("natural".to_string(), ModeConfig {
            name: "Natural".to_string(),
            description: "Envoi avec timing naturel et pauses intelligentes".to_string(),
            batch_size: 50,
            delay_between_batches: 300, // 5 minutes
            delay_variation: 60,        // ±1 minute
            max_emails_per_hour: 200,
            threading_mode: "conservative".to_string(),
            retry_attempts: 3,
            retry_delay: 30,
            rate_limiting: true,
            smart_throttling: true,
        });
        
        // Mode rapide
        sending_modes.insert("fast".to_string(), ModeConfig {
            name: "Fast".to_string(),
            description: "Envoi rapide avec sécurité maintenue".to_string(),
            batch_size: 100,
            delay_between_batches: 120, // 2 minutes
            delay_variation: 30,
            max_emails_per_hour: 500,
            threading_mode: "aggressive".to_string(),
            retry_attempts: 2,
            retry_delay: 15,
            rate_limiting: true,
            smart_throttling: true,
        });
        
        // Mode ultra-rapide (risqué)
        sending_modes.insert("turbo".to_string(), ModeConfig {
            name: "Turbo".to_string(),
            description: "Envoi très rapide - À utiliser avec précaution".to_string(),
            batch_size: 200,
            delay_between_batches: 60,  // 1 minute
            delay_variation: 15,
            max_emails_per_hour: 1000,
            threading_mode: "maximum".to_string(),
            retry_attempts: 1,
            retry_delay: 5,
            rate_limiting: false,
            smart_throttling: false,
        });
        
        Self {
            general: GeneralConfig {
                app_name: "Ultra Email Sender".to_string(),
                version: "1.0.0".to_string(),
                environment: "production".to_string(),
                timezone: "Europe/Paris".to_string(),
                default_language: "fr".to_string(),
                daily_email_limit: 10000,
                default_threads: 4,
                default_timeout: 30,
            },
            smtp_servers: vec![],
            sending_modes,
            rotation: RotationConfig {
                rotate_sender_name: true,
                rotate_subject: true,
                cc_enabled: false,
                cc_count_min: 1,
                cc_count_max: 3,
                cc_rotation_auto: true,
                cc_emails_pool: vec![],
                smtp_rotation_mode: "smart".to_string(),
            },
            security: SecurityConfig::default(),
            files: FilesConfig {
                templates_dir: "templates".to_string(),
                logs_dir: "logs".to_string(),
                data_dir: "data".to_string(),
                backups_dir: "backups".to_string(),
                default_template: "templates/default.html".to_string(),
                database_file: Some("data/emails.db".to_string()),
            },
            variables: VariablesConfig {
                global_variables: HashMap::new(),
                generators: HashMap::new(),
                date_format: "%d/%m/%Y".to_string(),
                time_format: "%H:%M".to_string(),
            },
            headers: HeadersConfig {
                headers_count: 200,
                header_types: vec![
                    "authentication".to_string(),
                    "security".to_string(),
                    "client".to_string(),
                    "server".to_string(),
                    "microsoft".to_string(),
                    "google".to_string(),
                    "apple".to_string(),
                    "tracking".to_string(),
                    "custom".to_string(),
                ],
                custom_headers: HashMap::new(),
                rotate_user_agents: true,
                ip_pool: vec![
                    "192.168.1.100".to_string(),
                    "10.0.0.25".to_string(),
                    "172.16.0.50".to_string(),
                ],
            },
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_port: 9090,
                collection_interval: 60,
                log_retention_days: 30,
                email_alerts: false,
                webhook_url: None,
            },
        }
    }
}