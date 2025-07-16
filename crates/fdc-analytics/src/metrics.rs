//! Analytics metrics collection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 分析指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    /// 处理的数据点数
    pub data_points_processed: u64,
    /// 计算的指标数
    pub indicators_calculated: u64,
    /// 生成的预测数
    pub predictions_generated: u64,
    /// 风险计算次数
    pub risk_calculations: u64,
    /// 平均处理时间（微秒）
    pub avg_processing_time_us: f64,
    /// 系统吞吐量（数据点/秒）
    pub throughput_points_per_sec: f64,
    /// 按类型统计
    pub type_stats: HashMap<String, u64>,
}

impl AnalyticsMetrics {
    /// 记录数据点处理
    pub fn record_data_point(&mut self, data_type: &str) {
        self.data_points_processed += 1;
        *self.type_stats.entry(data_type.to_string()).or_insert(0) += 1;
    }
    
    /// 记录指标计算
    pub fn record_indicator_calculation(&mut self) {
        self.indicators_calculated += 1;
    }
    
    /// 记录预测生成
    pub fn record_prediction(&mut self) {
        self.predictions_generated += 1;
    }
    
    /// 记录风险计算
    pub fn record_risk_calculation(&mut self) {
        self.risk_calculations += 1;
    }
    
    /// 记录处理时间
    pub fn record_processing_time(&mut self, time_us: u64) {
        let total_operations = self.data_points_processed + self.indicators_calculated + 
                              self.predictions_generated + self.risk_calculations;
        
        if total_operations > 0 {
            self.avg_processing_time_us = 
                (self.avg_processing_time_us * (total_operations - 1) as f64 + time_us as f64) 
                / total_operations as f64;
        }
    }
    
    /// 计算吞吐量
    pub fn calculate_throughput(&mut self, time_window_secs: f64) {
        if time_window_secs > 0.0 {
            self.throughput_points_per_sec = self.data_points_processed as f64 / time_window_secs;
        }
    }
}

/// 指标收集器
pub struct MetricsCollector {
    metrics: Arc<RwLock<AnalyticsMetrics>>,
    start_time: std::time::Instant,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(AnalyticsMetrics::default())),
            start_time: std::time::Instant::now(),
        }
    }
    
    /// 记录数据点处理
    pub async fn record_data_point(&self, data_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.record_data_point(data_type);
    }
    
    /// 记录指标计算
    pub async fn record_indicator_calculation(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.record_indicator_calculation();
    }
    
    /// 记录预测生成
    pub async fn record_prediction(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.record_prediction();
    }
    
    /// 记录风险计算
    pub async fn record_risk_calculation(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.record_risk_calculation();
    }
    
    /// 获取指标
    pub async fn get_metrics(&self) -> AnalyticsMetrics {
        let mut metrics = self.metrics.read().await.clone();
        
        // 计算吞吐量
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        metrics.calculate_throughput(elapsed_secs);
        
        metrics
    }
    
    /// 重置指标
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = AnalyticsMetrics::default();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_metrics() {
        let mut metrics = AnalyticsMetrics::default();
        
        metrics.record_data_point("market_data");
        metrics.record_data_point("trade_data");
        metrics.record_indicator_calculation();
        metrics.record_prediction();
        metrics.record_risk_calculation();
        
        assert_eq!(metrics.data_points_processed, 2);
        assert_eq!(metrics.indicators_calculated, 1);
        assert_eq!(metrics.predictions_generated, 1);
        assert_eq!(metrics.risk_calculations, 1);
        assert_eq!(metrics.type_stats.len(), 2);
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        collector.record_data_point("market_data").await;
        collector.record_indicator_calculation().await;
        collector.record_prediction().await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.data_points_processed, 1);
        assert_eq!(metrics.indicators_calculated, 1);
        assert_eq!(metrics.predictions_generated, 1);
    }
}
