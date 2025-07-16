//! Query engine configuration

use crate::cache::CachePolicy;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 查询配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    /// 查询超时时间
    pub query_timeout: Duration,
    /// 最大并发查询数
    pub max_concurrent_queries: usize,
    /// 最大结果集大小
    pub max_result_size: usize,
    /// 是否启用查询缓存
    pub enable_cache: bool,
    /// 缓存容量
    pub cache_capacity: usize,
    /// 缓存策略
    pub cache_policy: CachePolicy,
    /// 是否启用查询优化
    pub enable_optimization: bool,
    /// 是否启用指标收集
    pub enable_metrics: bool,
    /// 是否启用查询日志
    pub enable_query_log: bool,
    /// 慢查询阈值（毫秒）
    pub slow_query_threshold_ms: u64,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            query_timeout: Duration::from_secs(crate::DEFAULT_QUERY_TIMEOUT_SECS),
            max_concurrent_queries: crate::DEFAULT_MAX_CONCURRENT_QUERIES,
            max_result_size: crate::DEFAULT_MAX_RESULT_SIZE,
            enable_cache: true,
            cache_capacity: crate::DEFAULT_CACHE_SIZE,
            cache_policy: CachePolicy::LRU,
            enable_optimization: true,
            enable_metrics: true,
            enable_query_log: false,
            slow_query_threshold_ms: 1000,
        }
    }
}

impl QueryConfig {
    /// 创建新的查询配置
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 设置查询超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = timeout;
        self
    }
    
    /// 设置最大并发查询数
    pub fn with_max_concurrent_queries(mut self, max: usize) -> Self {
        self.max_concurrent_queries = max;
        self
    }
    
    /// 设置缓存配置
    pub fn with_cache(mut self, capacity: usize, policy: CachePolicy) -> Self {
        self.enable_cache = true;
        self.cache_capacity = capacity;
        self.cache_policy = policy;
        self
    }
    
    /// 禁用缓存
    pub fn without_cache(mut self) -> Self {
        self.enable_cache = false;
        self
    }
    
    /// 启用查询日志
    pub fn with_query_log(mut self, slow_threshold_ms: u64) -> Self {
        self.enable_query_log = true;
        self.slow_query_threshold_ms = slow_threshold_ms;
        self
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.query_timeout.as_secs() == 0 {
            return Err("Query timeout must be greater than 0".to_string());
        }
        
        if self.max_concurrent_queries == 0 {
            return Err("Max concurrent queries must be greater than 0".to_string());
        }
        
        if self.max_result_size == 0 {
            return Err("Max result size must be greater than 0".to_string());
        }
        
        if self.enable_cache && self.cache_capacity == 0 {
            return Err("Cache capacity must be greater than 0 when cache is enabled".to_string());
        }
        
        Ok(())
    }
    
    /// 生成配置摘要
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str("Query Configuration Summary:\n");
        summary.push_str(&format!("  Query Timeout: {:?}\n", self.query_timeout));
        summary.push_str(&format!("  Max Concurrent Queries: {}\n", self.max_concurrent_queries));
        summary.push_str(&format!("  Max Result Size: {} bytes\n", self.max_result_size));
        summary.push_str(&format!("  Cache: {} (capacity: {}, policy: {:?})\n", 
            self.enable_cache, self.cache_capacity, self.cache_policy));
        summary.push_str(&format!("  Optimization: {}\n", self.enable_optimization));
        summary.push_str(&format!("  Metrics: {}\n", self.enable_metrics));
        summary.push_str(&format!("  Query Log: {} (slow threshold: {}ms)\n", 
            self.enable_query_log, self.slow_query_threshold_ms));
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QueryConfig::default();
        assert!(config.enable_cache);
        assert!(config.enable_optimization);
        assert!(config.enable_metrics);
        assert!(!config.enable_query_log);
    }

    #[test]
    fn test_config_validation() {
        let config = QueryConfig::default();
        assert!(config.validate().is_ok());
        
        let mut invalid_config = config.clone();
        invalid_config.max_concurrent_queries = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = QueryConfig::new()
            .with_timeout(Duration::from_secs(60))
            .with_max_concurrent_queries(200)
            .with_cache(1024, CachePolicy::LRU)
            .with_query_log(500);
        
        assert_eq!(config.query_timeout, Duration::from_secs(60));
        assert_eq!(config.max_concurrent_queries, 200);
        assert!(config.enable_cache);
        assert!(config.enable_query_log);
        assert_eq!(config.slow_query_threshold_ms, 500);
    }

    #[test]
    fn test_config_summary() {
        let config = QueryConfig::default();
        let summary = config.summary();
        
        assert!(summary.contains("Query Configuration Summary"));
        assert!(summary.contains("Query Timeout"));
        assert!(summary.contains("Cache"));
        assert!(summary.contains("Optimization"));
    }
}
