//! Risk calculation engine

use crate::{config::RiskConfig, models::RiskMetrics};
use fdc_core::error::Result;
use serde::{Deserialize, Serialize};

/// 风险引擎统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskStats {
    /// 风险计算次数
    pub calculations_performed: u64,
    /// 平均计算时间（毫秒）
    pub avg_calculation_time_ms: f64,
}

/// 风险引擎
pub struct RiskEngine {
    config: RiskConfig,
    stats: RiskStats,
}

impl RiskEngine {
    /// 创建新的风险引擎
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            stats: RiskStats::default(),
        }
    }
    
    /// 计算风险指标
    pub async fn calculate_risk_metrics(&mut self, returns: Vec<f64>) -> Result<RiskMetrics> {
        // 简化实现
        let start_time = std::time::Instant::now();
        
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let volatility = variance.sqrt();
        
        // 简化的VaR计算（正态分布假设）
        let var = mean_return - 1.645 * volatility; // 95% VaR
        let cvar = mean_return - 2.0 * volatility; // 简化的CVaR
        
        let metrics = RiskMetrics {
            var,
            cvar,
            max_drawdown: 0.1, // 简化值
            sharpe_ratio: mean_return / volatility,
            volatility,
            beta: None,
        };
        
        let calculation_time = start_time.elapsed().as_millis() as f64;
        self.stats.calculations_performed += 1;
        self.stats.avg_calculation_time_ms = 
            (self.stats.avg_calculation_time_ms * (self.stats.calculations_performed - 1) as f64 + calculation_time) 
            / self.stats.calculations_performed as f64;
        
        Ok(metrics)
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> &RiskStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_risk_engine() {
        let config = RiskConfig::default();
        let mut engine = RiskEngine::new(config);
        
        let returns = vec![0.01, -0.02, 0.03, -0.01, 0.02];
        let result = engine.calculate_risk_metrics(returns).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.volatility > 0.0);
        assert!(metrics.var < 0.0); // VaR应该是负值
        
        let stats = engine.get_stats();
        assert_eq!(stats.calculations_performed, 1);
    }
}
