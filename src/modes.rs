use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum SendingMode {
    Natural,
    Balanced,
    Fast,
    Turbo,
    Custom(String),
}

impl SendingMode {
    pub fn from_string(mode: &str) -> Result<Self> {
        match mode.to_lowercase().as_str() {
            "natural" => Ok(SendingMode::Natural),
            "balanced" => Ok(SendingMode::Balanced),
            "fast" => Ok(SendingMode::Fast),
            "turbo" => Ok(SendingMode::Turbo),
            custom => Ok(SendingMode::Custom(custom.to_string())),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            SendingMode::Natural => "natural".to_string(),
            SendingMode::Balanced => "balanced".to_string(),
            SendingMode::Fast => "fast".to_string(),
            SendingMode::Turbo => "turbo".to_string(),
            SendingMode::Custom(name) => name.clone(),
        }
    }
    
    pub fn description(&self) -> &str {
        match self {
            SendingMode::Natural => "Mode naturel - Timing humain ultra-sécurisé",
            SendingMode::Balanced => "Mode équilibré - Bon compromis vitesse/sécurité",
            SendingMode::Fast => "Mode rapide - Surveillance recommandée",
            SendingMode::Turbo => "Mode turbo - Experts seulement",
            SendingMode::Custom(_) => "Mode personnalisé",
        }
    }
}