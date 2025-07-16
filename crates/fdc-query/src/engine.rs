//! Main query engine implementation

use crate::{
    parser::{SqlParser, ParsedQuery},
    optimizer::{QueryOptimizer, OptimizedPlan},
    executor::{QueryExecutor, DefaultQueryExecutor, ExecutionContext, ExecutionResult},
    planner::{QueryPlanner, ExecutionPlan},
    cache::{QueryCache, CachePolicy},
    metrics::QueryMetrics,
};
use fdc_core::error::{Error, Result};
use fdc_storage::engine::StorageEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// 查询引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEngineConfig {
    /// 是否启用查询缓存
    pub enable_cache: bool,
    /// 缓存容量
    pub cache_capacity: usize,
    /// 缓存策略
    pub cache_policy: CachePolicy,
    /// 查询超时时间
    pub query_timeout: Duration,
    /// 最大并发查询数
    pub max_concurrent_queries: usize,
    /// 最大结果集大小
    pub max_result_size: usize,
    /// 是否启用查询优化
    pub enable_optimization: bool,
    /// 是否启用指标收集
    pub enable_metrics: bool,
}

impl Default for QueryEngineConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_capacity: crate::DEFAULT_CACHE_SIZE,
            cache_policy: CachePolicy::LRU,
            query_timeout: Duration::from_secs(crate::DEFAULT_QUERY_TIMEOUT_SECS),
            max_concurrent_queries: crate::DEFAULT_MAX_CONCURRENT_QUERIES,
            max_result_size: crate::DEFAULT_MAX_RESULT_SIZE,
            enable_optimization: true,
            enable_metrics: true,
        }
    }
}

/// 查询引擎
pub struct QueryEngine {
    /// 配置
    config: QueryEngineConfig,
    /// SQL解析器
    parser: SqlParser,
    /// 查询优化器
    optimizer: Arc<RwLock<QueryOptimizer>>,
    /// 查询计划器
    planner: QueryPlanner,
    /// 查询执行器
    executor: Arc<dyn QueryExecutor>,
    /// 查询缓存
    cache: Arc<RwLock<QueryCache>>,
    /// 查询指标
    metrics: Arc<RwLock<QueryMetrics>>,
}

impl QueryEngine {
    /// 创建新的查询引擎
    pub fn new(storage_engine: Arc<dyn StorageEngine>, config: QueryEngineConfig) -> Self {
        let cache = Arc::new(RwLock::new(QueryCache::new(
            config.cache_policy.clone(),
            config.cache_capacity,
        )));
        
        let metrics = Arc::new(RwLock::new(QueryMetrics::new()));
        
        Self {
            config,
            parser: SqlParser::new(),
            optimizer: Arc::new(RwLock::new(QueryOptimizer::new())),
            planner: QueryPlanner::new(),
            executor: Arc::new(DefaultQueryExecutor::new(storage_engine)),
            cache,
            metrics,
        }
    }
    
    /// 执行SQL查询
    pub async fn execute_sql(&self, sql: &str) -> Result<ExecutionResult> {
        let query_id = uuid::Uuid::new_v4().to_string();
        let context = ExecutionContext::new(query_id.clone())
            .with_timeout(self.config.query_timeout);
        
        self.execute_sql_with_context(sql, context).await
    }
    
    /// 使用上下文执行SQL查询
    pub async fn execute_sql_with_context(&self, sql: &str, context: ExecutionContext) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        
        // 记录查询开始
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.record_query_start();
        }
        
        // 检查缓存
        if self.config.enable_cache {
            let query_hash = self.calculate_query_hash(sql);
            let mut cache = self.cache.write().await;
            if let Some(cached_result) = cache.get(&query_hash) {
                // 记录缓存命中
                if self.config.enable_metrics {
                    let mut metrics = self.metrics.write().await;
                    metrics.record_cache_hit();
                }
                return Ok(cached_result);
            }
        }
        
        // 解析SQL
        let parsed_query = self.parser.parse(sql)?;
        
        // 优化查询
        let optimized_plan = if self.config.enable_optimization {
            let mut optimizer = self.optimizer.write().await;
            optimizer.optimize(parsed_query)?
        } else {
            OptimizedPlan::new(parsed_query)
        };
        
        // 创建执行计划
        let _execution_plan = self.planner.create_plan(&optimized_plan)?;
        
        // 执行查询
        let result = self.executor.execute(optimized_plan, context).await?;
        
        // 缓存结果
        if self.config.enable_cache && result.is_success() {
            let query_hash = self.calculate_query_hash(sql);
            let mut cache = self.cache.write().await;
            cache.put(query_hash, result.clone());
        }
        
        // 记录查询完成
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            let execution_time = start_time.elapsed();
            metrics.record_query_complete(execution_time, result.is_success());
        }
        
        Ok(result)
    }
    
    /// 验证SQL语法
    pub fn validate_sql(&self, sql: &str) -> Result<()> {
        self.parser.validate(sql)
    }
    
    /// 解析SQL查询
    pub fn parse_sql(&self, sql: &str) -> Result<ParsedQuery> {
        self.parser.parse(sql)
    }
    
    /// 格式化SQL
    pub fn format_sql(&self, sql: &str) -> Result<String> {
        self.parser.format(sql)
    }
    
    /// 获取查询计划
    pub async fn explain_query(&self, sql: &str) -> Result<ExecutionPlan> {
        let parsed_query = self.parser.parse(sql)?;
        let optimized_plan = if self.config.enable_optimization {
            let mut optimizer = self.optimizer.write().await;
            optimizer.optimize(parsed_query)?
        } else {
            OptimizedPlan::new(parsed_query)
        };
        
        self.planner.create_plan(&optimized_plan)
    }
    
    /// 取消查询
    pub async fn cancel_query(&self, query_id: &str) -> Result<()> {
        self.executor.cancel(query_id).await
    }
    
    /// 获取查询统计信息
    pub async fn get_query_stats(&self) -> Result<QueryMetrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }
    
    /// 获取缓存统计信息
    pub async fn get_cache_stats(&self) -> Result<crate::cache::CacheStats> {
        let cache = self.cache.read().await;
        Ok(cache.stats().clone())
    }
    
    /// 清除查询缓存
    pub async fn clear_cache(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.clear();
        Ok(())
    }
    
    /// 计算查询哈希
    fn calculate_query_hash(&self, sql: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// 获取配置
    pub fn config(&self) -> &QueryEngineConfig {
        &self.config
    }
    
    /// 更新配置
    pub fn update_config(&mut self, config: QueryEngineConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fdc_storage::engines::memory::MemoryEngine;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_query_engine_creation() {
        let memory_engine = MemoryEngine::new(HashMap::new()).await.unwrap();
        let config = QueryEngineConfig::default();
        let engine = QueryEngine::new(Arc::new(memory_engine), config);
        
        assert!(engine.config().enable_cache);
        assert!(engine.config().enable_optimization);
    }

    #[tokio::test]
    async fn test_sql_validation() {
        let memory_engine = MemoryEngine::new(HashMap::new()).await.unwrap();
        let config = QueryEngineConfig::default();
        let engine = QueryEngine::new(Arc::new(memory_engine), config);
        
        assert!(engine.validate_sql("SELECT * FROM users").is_ok());
        assert!(engine.validate_sql("INVALID SQL").is_err());
    }

    #[tokio::test]
    async fn test_sql_execution() {
        let memory_engine = MemoryEngine::new(HashMap::new()).await.unwrap();
        let config = QueryEngineConfig::default();
        let engine = QueryEngine::new(Arc::new(memory_engine), config);
        
        let result = engine.execute_sql("SELECT * FROM users").await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_query_caching() {
        let memory_engine = MemoryEngine::new(HashMap::new()).await.unwrap();
        let config = QueryEngineConfig::default();
        let engine = QueryEngine::new(Arc::new(memory_engine), config);
        
        // 第一次执行
        let result1 = engine.execute_sql("SELECT * FROM users").await.unwrap();
        
        // 第二次执行（应该命中缓存）
        let result2 = engine.execute_sql("SELECT * FROM users").await.unwrap();
        
        assert!(result1.is_success());
        assert!(result2.is_success());
        
        let cache_stats = engine.get_cache_stats().await.unwrap();
        assert!(cache_stats.hits > 0);
    }

    #[tokio::test]
    async fn test_explain_query() {
        let memory_engine = MemoryEngine::new(HashMap::new()).await.unwrap();
        let config = QueryEngineConfig::default();
        let engine = QueryEngine::new(Arc::new(memory_engine), config);
        
        let plan = engine.explain_query("SELECT * FROM users WHERE id = 1").await.unwrap();
        assert!(plan.estimated_cost > 0.0);
    }
}
