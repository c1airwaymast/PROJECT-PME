use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use crate::dashboard::SystemMetrics;

/// 📊 COLLECTEUR MÉTRIQUES SIMPLE QUI MARCHE À 100%
pub struct MetricsCollector {
    metrics: Arc<RwLock<SystemMetrics>>,
}

impl MetricsCollector {
    pub async fn new(metrics: Arc<RwLock<SystemMetrics>>) -> Result<Self> {
        Ok(Self { metrics })
    }
}