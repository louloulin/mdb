//! Query result caching system

use crate::executor::ExecutionResult;
use fdc_core::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 缓存策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CachePolicy {
    /// 最近最少使用
    LRU,
    /// 最不经常使用
    LFU,
    /// 先进先出
    FIFO,
    /// 基于TTL
    TTL(Duration),
}

/// 缓存统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
    pub capacity: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    result: ExecutionResult,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
}

/// 查询缓存
pub struct QueryCache {
    policy: CachePolicy,
    capacity: usize,
    entries: HashMap<String, CacheEntry>,
    stats: CacheStats,
}

impl QueryCache {
    pub fn new(policy: CachePolicy, capacity: usize) -> Self {
        Self {
            policy,
            capacity,
            entries: HashMap::new(),
            stats: CacheStats { capacity, ..Default::default() },
        }
    }
    
    pub fn get(&mut self, query_hash: &str) -> Option<ExecutionResult> {
        if let Some(entry) = self.entries.get_mut(query_hash) {
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            self.stats.hits += 1;
            Some(entry.result.clone())
        } else {
            self.stats.misses += 1;
            None
        }
    }
    
    pub fn put(&mut self, query_hash: String, result: ExecutionResult) {
        if self.entries.len() >= self.capacity {
            self.evict_one();
        }
        
        let entry = CacheEntry {
            result,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
        };
        
        self.entries.insert(query_hash, entry);
        self.stats.size = self.entries.len();
    }
    
    fn evict_one(&mut self) {
        if let Some(key) = self.entries.keys().next().cloned() {
            self.entries.remove(&key);
            self.stats.evictions += 1;
        }
    }
    
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats.size = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = QueryCache::new(CachePolicy::LRU, 2);
        
        let result = ExecutionResult::success(Vec::new(), 1000);
        cache.put("query1".to_string(), result.clone());
        
        assert_eq!(cache.get("query1").unwrap().execution_time_us, 1000);
        assert_eq!(cache.get("query2"), None);
        
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);
    }
}
