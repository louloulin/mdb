//! Query metrics collection

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// 查询指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryMetrics {
    /// 总查询数
    pub total_queries: u64,
    /// 成功查询数
    pub successful_queries: u64,
    /// 失败查询数
    pub failed_queries: u64,
    /// 缓存命中数
    pub cache_hits: u64,
    /// 缓存未命中数
    pub cache_misses: u64,
    /// 平均执行时间（微秒）
    pub avg_execution_time_us: f64,
    /// 最小执行时间（微秒）
    pub min_execution_time_us: u64,
    /// 最大执行时间（微秒）
    pub max_execution_time_us: u64,
    /// 当前并发查询数
    pub concurrent_queries: u64,
    /// 启动时间
    pub start_time: Option<SystemTime>,
}

impl QueryMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Some(SystemTime::now()),
            ..Default::default()
        }
    }
    
    pub fn record_query_start(&mut self) {
        self.total_queries += 1;
        self.concurrent_queries += 1;
    }
    
    pub fn record_query_complete(&mut self, execution_time: Duration, success: bool) {
        self.concurrent_queries = self.concurrent_queries.saturating_sub(1);
        
        let execution_time_us = execution_time.as_micros() as u64;
        
        if success {
            self.successful_queries += 1;
        } else {
            self.failed_queries += 1;
        }
        
        // 更新执行时间统计
        if self.min_execution_time_us == 0 || execution_time_us < self.min_execution_time_us {
            self.min_execution_time_us = execution_time_us;
        }
        
        if execution_time_us > self.max_execution_time_us {
            self.max_execution_time_us = execution_time_us;
        }
        
        // 更新平均执行时间
        let total_completed = self.successful_queries + self.failed_queries;
        if total_completed > 0 {
            self.avg_execution_time_us = (self.avg_execution_time_us * (total_completed - 1) as f64 + execution_time_us as f64) / total_completed as f64;
        }
    }
    
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }
    
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }
    
    pub fn success_rate(&self) -> f64 {
        let total_completed = self.successful_queries + self.failed_queries;
        if total_completed == 0 {
            0.0
        } else {
            self.successful_queries as f64 / total_completed as f64
        }
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        let total_cache_requests = self.cache_hits + self.cache_misses;
        if total_cache_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_cache_requests as f64
        }
    }
    
    pub fn queries_per_second(&self) -> f64 {
        if let Some(start_time) = self.start_time {
            if let Ok(uptime) = SystemTime::now().duration_since(start_time) {
                let seconds = uptime.as_secs_f64();
                if seconds > 0.0 {
                    return self.total_queries as f64 / seconds;
                }
            }
        }
        0.0
    }
    
    pub fn uptime(&self) -> Option<Duration> {
        self.start_time.and_then(|start| SystemTime::now().duration_since(start).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_query_metrics() {
        let mut metrics = QueryMetrics::new();
        
        metrics.record_query_start();
        metrics.record_query_complete(Duration::from_millis(100), true);
        
        assert_eq!(metrics.total_queries, 1);
        assert_eq!(metrics.successful_queries, 1);
        assert_eq!(metrics.concurrent_queries, 0);
        assert_eq!(metrics.success_rate(), 1.0);
    }

    #[test]
    fn test_cache_metrics() {
        let mut metrics = QueryMetrics::new();
        
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.cache_hit_rate(), 0.5);
    }
}
