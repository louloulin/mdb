//! Data aggregation engine

use crate::{config::AggregationConfig, models::AggregationResult};
use fdc_core::error::Result;
use std::collections::HashMap;

/// 聚合引擎
pub struct AggregationEngine {
    config: AggregationConfig,
}

impl AggregationEngine {
    /// 创建新的聚合引擎
    pub fn new(config: AggregationConfig) -> Self {
        Self { config }
    }
    
    /// 执行聚合操作
    pub async fn aggregate(&self, data: Vec<HashMap<String, f64>>) -> Result<Vec<AggregationResult>> {
        // 简化实现：按配置的分组字段进行聚合
        let mut results = Vec::new();
        
        if data.is_empty() {
            return Ok(results);
        }
        
        // 简化的聚合逻辑
        let mut aggregated_values = HashMap::new();
        for function in &self.config.functions {
            match function.as_str() {
                "sum" => {
                    let sum: f64 = data.iter()
                        .filter_map(|row| row.get("value"))
                        .sum();
                    aggregated_values.insert("sum".to_string(), sum);
                }
                "avg" => {
                    let values: Vec<f64> = data.iter()
                        .filter_map(|row| row.get("value"))
                        .cloned()
                        .collect();
                    if !values.is_empty() {
                        let avg = values.iter().sum::<f64>() / values.len() as f64;
                        aggregated_values.insert("avg".to_string(), avg);
                    }
                }
                "count" => {
                    aggregated_values.insert("count".to_string(), data.len() as f64);
                }
                _ => {}
            }
        }
        
        let result = AggregationResult {
            group_key: HashMap::new(),
            aggregated_values,
            count: data.len() as u64,
            window_start: chrono::Utc::now() - self.config.window_size,
            window_end: chrono::Utc::now(),
        };
        
        results.push(result);
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aggregation() {
        let config = AggregationConfig::default();
        let engine = AggregationEngine::new(config);
        
        let mut data = Vec::new();
        for i in 1..=5 {
            let mut row = HashMap::new();
            row.insert("value".to_string(), i as f64);
            data.push(row);
        }
        
        let results = engine.aggregate(data).await.unwrap();
        assert_eq!(results.len(), 1);
        
        let result = &results[0];
        assert_eq!(result.count, 5);
        assert_eq!(result.aggregated_values.get("sum"), Some(&15.0));
        assert_eq!(result.aggregated_values.get("avg"), Some(&3.0));
        assert_eq!(result.aggregated_values.get("count"), Some(&5.0));
    }
}
