//! Error recovery and resilience management

use crate::config::RecoveryConfig;
// use fdc_core::error::{Error, Result}; // 暂时未使用
use serde::{Deserialize, Serialize};

/// 恢复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// 立即重试
    Immediate,
    /// 固定间隔重试
    FixedInterval,
    /// 指数退避
    ExponentialBackoff,
    /// 自定义策略
    Custom(String),
}

/// 恢复管理器
pub struct RecoveryManager {
    config: RecoveryConfig,
}

impl RecoveryManager {
    pub fn new(config: RecoveryConfig) -> Self {
        Self { config }
    }
    
    pub async fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.config.max_retries
    }
    
    pub async fn get_retry_delay(&self, attempt: u32) -> std::time::Duration {
        match self.config.recovery_strategy.as_str() {
            "exponential_backoff" => {
                let base_delay = self.config.retry_interval.as_millis() as u64;
                let delay = base_delay * 2_u64.pow(attempt);
                std::time::Duration::from_millis(delay)
            }
            _ => self.config.retry_interval,
        }
    }
}
