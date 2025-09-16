use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModeConfig {
    /// Nom du mode
    pub name: String,
    
    /// Description du mode
    pub description: String,
    
    /// Taille des batches BCC
    pub batch_size: usize,
    
    /// Délai entre batches (secondes)
    pub delay_between_batches: u64,
    
    /// Variation aléatoire du délai (±secondes)
    pub delay_variation: u64,
    
    /// Maximum d'emails par heure
    pub max_emails_per_hour: u32,
    
    /// Mode de threading (conservative, balanced, aggressive, maximum)
    pub threading_mode: String,
    
    /// Nombre de tentatives en cas d'échec
    pub retry_attempts: u32,
    
    /// Délai entre tentatives (secondes)
    pub retry_delay: u64,
    
    /// Activer le rate limiting
    pub rate_limiting: bool,
    
    /// Throttling intelligent selon les réponses
    pub smart_throttling: bool,
}

impl ModeConfig {
    pub fn get_thread_count(&self) -> usize {
        match self.threading_mode.as_str() {
            "conservative" => 2,
            "balanced" => 4,
            "aggressive" => 8,
            "maximum" => 16,
            _ => 4, // Par défaut
        }
    }
    
    pub fn get_actual_delay(&self) -> u64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let variation = rng.gen_range(0..=self.delay_variation * 2) as i64 - self.delay_variation as i64;
        (self.delay_between_batches as i64 + variation).max(1) as u64
    }
    
    pub fn is_rate_limit_exceeded(&self, emails_sent_last_hour: u32) -> bool {
        emails_sent_last_hour >= self.max_emails_per_hour
    }
    
    pub fn get_recommended_batch_delay(&self, current_success_rate: f32) -> u64 {
        if !self.smart_throttling {
            return self.get_actual_delay();
        }
        
        // Ajuster le délai selon le taux de succès
        let base_delay = self.delay_between_batches as f32;
        let adjusted_delay = match current_success_rate {
            rate if rate >= 0.95 => base_delay * 0.8,  // Accélérer si très bon taux
            rate if rate >= 0.90 => base_delay,        // Normal
            rate if rate >= 0.80 => base_delay * 1.2,  // Ralentir un peu
            rate if rate >= 0.70 => base_delay * 1.5,  // Ralentir plus
            _ => base_delay * 2.0,                     // Ralentir beaucoup
        };
        
        adjusted_delay as u64
    }
}