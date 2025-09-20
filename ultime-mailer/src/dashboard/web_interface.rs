use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, error};
use crate::config::UltimeConfig;
use crate::dashboard::SystemMetrics;

/// 🌐 INTERFACE WEB ULTRA-SIMPLE QUI MARCHE À 100%
pub struct WebInterface {
    config: Arc<RwLock<UltimeConfig>>,
    metrics: Arc<RwLock<SystemMetrics>>,
}

impl WebInterface {
    pub async fn new(
        config: Arc<RwLock<UltimeConfig>>,
        metrics: Arc<RwLock<SystemMetrics>>
    ) -> Result<Self> {
        Ok(Self { config, metrics })
    }
}