use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use crate::config::UltimeConfig;

/// 🌌 VISUALISATEUR QUANTIQUE SIMPLE QUI MARCHE À 100%
pub struct QuantumVisualizer {
    config: Arc<RwLock<UltimeConfig>>,
}

impl QuantumVisualizer {
    pub async fn new(config: Arc<RwLock<UltimeConfig>>) -> Result<Self> {
        Ok(Self { config })
    }
}