//! Backpressure control for flow management

use crate::config::BackpressureConfig;
// use serde::{Deserialize, Serialize}; // 暂时未使用
use std::sync::Arc;
use tokio::sync::RwLock;

/// 背压控制器
pub struct BackpressureController {
    config: BackpressureConfig,
    current_load: Arc<RwLock<f64>>,
    is_active: Arc<RwLock<bool>>,
}

impl BackpressureController {
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            config,
            current_load: Arc::new(RwLock::new(0.0)),
            is_active: Arc::new(RwLock::new(false)),
        }
    }
    
    pub async fn update_load(&self, load: f64) {
        let mut current_load = self.current_load.write().await;
        *current_load = load;
        
        let mut is_active = self.is_active.write().await;
        if load > self.config.threshold {
            *is_active = true;
        } else if load < self.config.recovery_threshold {
            *is_active = false;
        }
    }
    
    pub async fn should_apply_backpressure(&self) -> bool {
        *self.is_active.read().await
    }
    
    pub async fn get_current_load(&self) -> f64 {
        *self.current_load.read().await
    }
}
