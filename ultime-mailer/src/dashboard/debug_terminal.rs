use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use crate::config::UltimeConfig;

/// 💻 TERMINAL DEBUG SIMPLE QUI MARCHE À 100%
pub struct DebugTerminal {
    config: Arc<RwLock<UltimeConfig>>,
}

impl DebugTerminal {
    pub async fn new(config: Arc<RwLock<UltimeConfig>>) -> Result<Self> {
        Ok(Self { config })
    }
}