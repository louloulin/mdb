//! Query executor for executing optimized queries

use crate::optimizer::OptimizedPlan;
use fdc_core::{error::{Error, Result}, types::Value};
use fdc_storage::engine::StorageEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;

/// 执行结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 结果行数
    pub rows: Vec<HashMap<String, Value>>,
    /// 执行时间（微秒）
    pub execution_time_us: u64,
    /// 影响的行数
    pub affected_rows: u64,
    /// 执行统计信息
    pub stats: ExecutionStats,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

impl ExecutionResult {
    /// 创建成功的执行结果
    pub fn success(rows: Vec<HashMap<String, Value>>, execution_time_us: u64) -> Self {
        Self {
            rows,
            execution_time_us,
            affected_rows: 0,
            stats: ExecutionStats::default(),
            error: None,
        }
    }
    
    /// 创建错误的执行结果
    pub fn error(error: String) -> Self {
        Self {
            rows: Vec::new(),
            execution_time_us: 0,
            affected_rows: 0,
            stats: ExecutionStats::default(),
            error: Some(error),
        }
    }
    
    /// 是否成功
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
    
    /// 获取行数
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
    
    /// 获取执行时间（毫秒）
    pub fn execution_time_ms(&self) -> f64 {
        self.execution_time_us as f64 / 1000.0
    }
}

/// 执行统计信息
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// 扫描的行数
    pub rows_scanned: u64,
    /// 过滤的行数
    pub rows_filtered: u64,
    /// 排序的行数
    pub rows_sorted: u64,
    /// 聚合的行数
    pub rows_aggregated: u64,
    /// 使用的内存（字节）
    pub memory_used: u64,
    /// 磁盘I/O次数
    pub disk_io_count: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
}

impl ExecutionStats {
    /// 获取缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }
    
    /// 获取过滤率
    pub fn filter_rate(&self) -> f64 {
        if self.rows_scanned > 0 {
            self.rows_filtered as f64 / self.rows_scanned as f64
        } else {
            0.0
        }
    }
}

/// 执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 查询ID
    pub query_id: String,
    /// 用户ID
    pub user_id: Option<String>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 查询参数
    pub parameters: HashMap<String, Value>,
    /// 执行超时时间
    pub timeout: Duration,
    /// 最大结果行数
    pub max_rows: Option<usize>,
    /// 是否启用缓存
    pub enable_cache: bool,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new(query_id: String) -> Self {
        Self {
            query_id,
            user_id: None,
            session_id: None,
            parameters: HashMap::new(),
            timeout: Duration::from_secs(30),
            max_rows: Some(10000),
            enable_cache: true,
        }
    }
    
    /// 设置用户ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// 设置会话ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
    
    /// 添加参数
    pub fn with_parameter(mut self, key: String, value: Value) -> Self {
        self.parameters.insert(key, value);
        self
    }
    
    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// 设置最大行数
    pub fn with_max_rows(mut self, max_rows: usize) -> Self {
        self.max_rows = Some(max_rows);
        self
    }
}

/// 查询执行器特征
#[async_trait]
pub trait QueryExecutor: Send + Sync {
    /// 执行查询
    async fn execute(&self, plan: OptimizedPlan, context: ExecutionContext) -> Result<ExecutionResult>;
    
    /// 取消查询
    async fn cancel(&self, query_id: &str) -> Result<()>;
    
    /// 获取执行统计信息
    async fn get_stats(&self) -> Result<HashMap<String, u64>>;
}

/// 默认查询执行器
pub struct DefaultQueryExecutor {
    /// 存储引擎
    storage_engine: Arc<dyn StorageEngine>,
    /// 正在执行的查询
    running_queries: Arc<dashmap::DashMap<String, Instant>>,
}

impl DefaultQueryExecutor {
    /// 创建新的查询执行器
    pub fn new(storage_engine: Arc<dyn StorageEngine>) -> Self {
        Self {
            storage_engine,
            running_queries: Arc::new(dashmap::DashMap::new()),
        }
    }
    
    /// 执行SELECT查询
    async fn execute_select(&self, plan: &OptimizedPlan, context: &ExecutionContext) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        // 记录查询开始
        self.running_queries.insert(context.query_id.clone(), start_time);
        
        // 简化的SELECT执行逻辑
        let mut rows = Vec::new();
        let mut stats = ExecutionStats::default();
        
        // 模拟数据扫描
        for table in &plan.original_query.tables {
            // 扫描表数据（简化实现）
            let scan_result = self.scan_table(table, &context.parameters).await?;
            rows.extend(scan_result);
            stats.rows_scanned += 100; // 模拟扫描100行
        }
        
        // 应用过滤条件
        if plan.original_query.sql.to_lowercase().contains("where") {
            let filtered_rows = self.apply_filters(&rows, &plan.original_query.sql)?;
            stats.rows_filtered = (rows.len() - filtered_rows.len()) as u64;
            rows = filtered_rows;
        }
        
        // 应用排序
        if plan.original_query.sql.to_lowercase().contains("order by") {
            rows = self.apply_sorting(rows)?;
            stats.rows_sorted = rows.len() as u64;
        }
        
        // 应用限制
        if let Some(max_rows) = context.max_rows {
            if rows.len() > max_rows {
                rows.truncate(max_rows);
            }
        }
        
        // 移除查询记录
        self.running_queries.remove(&context.query_id);
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        let mut result = ExecutionResult::success(rows, execution_time);
        result.stats = stats;
        
        Ok(result)
    }
    
    /// 扫描表数据
    async fn scan_table(&self, table: &str, _parameters: &HashMap<String, Value>) -> Result<Vec<HashMap<String, Value>>> {
        // 简化实现：返回模拟数据
        let mut rows = Vec::new();
        
        match table {
            "users" => {
                for i in 1..=10 {
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), Value::Int64(i));
                    row.insert("name".to_string(), Value::String(format!("User{}", i)));
                    row.insert("email".to_string(), Value::String(format!("user{}@example.com", i)));
                    rows.push(row);
                }
            }
            "orders" => {
                for i in 1..=20 {
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), Value::Int64(i));
                    row.insert("user_id".to_string(), Value::Int64((i % 10) + 1));
                    row.insert("amount".to_string(), Value::Float64(100.0 * i as f64));
                    rows.push(row);
                }
            }
            _ => {
                // 默认返回空结果
            }
        }
        
        Ok(rows)
    }
    
    /// 应用过滤条件
    fn apply_filters(&self, rows: &[HashMap<String, Value>], _sql: &str) -> Result<Vec<HashMap<String, Value>>> {
        // 简化实现：返回前一半数据（模拟过滤）
        let filtered_count = rows.len() / 2;
        Ok(rows.iter().take(filtered_count).cloned().collect())
    }
    
    /// 应用排序
    fn apply_sorting(&self, mut rows: Vec<HashMap<String, Value>>) -> Result<Vec<HashMap<String, Value>>> {
        // 简化实现：按第一个字段排序
        rows.sort_by(|a, b| {
            if let (Some(a_val), Some(b_val)) = (a.values().next(), b.values().next()) {
                match (a_val, b_val) {
                    (Value::Int64(a), Value::Int64(b)) => a.cmp(b),
                    (Value::String(a), Value::String(b)) => a.cmp(b),
                    _ => std::cmp::Ordering::Equal,
                }
            } else {
                std::cmp::Ordering::Equal
            }
        });
        
        Ok(rows)
    }
    
    /// 执行INSERT查询
    async fn execute_insert(&self, _plan: &OptimizedPlan, _context: &ExecutionContext) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        // 简化的INSERT执行逻辑
        let affected_rows = 1; // 模拟插入1行
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        let mut result = ExecutionResult::success(Vec::new(), execution_time);
        result.affected_rows = affected_rows;
        
        Ok(result)
    }
    
    /// 执行UPDATE查询
    async fn execute_update(&self, _plan: &OptimizedPlan, _context: &ExecutionContext) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        // 简化的UPDATE执行逻辑
        let affected_rows = 5; // 模拟更新5行
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        let mut result = ExecutionResult::success(Vec::new(), execution_time);
        result.affected_rows = affected_rows;
        
        Ok(result)
    }
    
    /// 执行DELETE查询
    async fn execute_delete(&self, _plan: &OptimizedPlan, _context: &ExecutionContext) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        
        // 简化的DELETE执行逻辑
        let affected_rows = 3; // 模拟删除3行
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        let mut result = ExecutionResult::success(Vec::new(), execution_time);
        result.affected_rows = affected_rows;
        
        Ok(result)
    }
}

#[async_trait]
impl QueryExecutor for DefaultQueryExecutor {
    async fn execute(&self, plan: OptimizedPlan, context: ExecutionContext) -> Result<ExecutionResult> {
        // 检查超时
        let start_time = Instant::now();
        
        let result = match plan.original_query.query_type {
            crate::parser::QueryType::Select => self.execute_select(&plan, &context).await,
            crate::parser::QueryType::Insert => self.execute_insert(&plan, &context).await,
            crate::parser::QueryType::Update => self.execute_update(&plan, &context).await,
            crate::parser::QueryType::Delete => self.execute_delete(&plan, &context).await,
            _ => Err(Error::unimplemented("Query type not supported")),
        };
        
        // 检查是否超时
        if start_time.elapsed() > context.timeout {
            return Ok(ExecutionResult::error("Query timeout".to_string()));
        }
        
        result
    }
    
    async fn cancel(&self, query_id: &str) -> Result<()> {
        self.running_queries.remove(query_id);
        Ok(())
    }
    
    async fn get_stats(&self) -> Result<HashMap<String, u64>> {
        let mut stats = HashMap::new();
        stats.insert("running_queries".to_string(), self.running_queries.len() as u64);
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedQuery, QueryType};
    use crate::optimizer::OptimizedPlan;
    use fdc_storage::engines::memory::MemoryEngine;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_execution_result() {
        let result = ExecutionResult::success(Vec::new(), 1000);
        assert!(result.is_success());
        assert_eq!(result.execution_time_ms(), 1.0);
    }

    #[test]
    fn test_execution_context() {
        let context = ExecutionContext::new("query_1".to_string())
            .with_user_id("user_1".to_string())
            .with_max_rows(100);
        
        assert_eq!(context.query_id, "query_1");
        assert_eq!(context.user_id, Some("user_1".to_string()));
        assert_eq!(context.max_rows, Some(100));
    }

    #[tokio::test]
    async fn test_select_execution() {
        let memory_engine = MemoryEngine::new(HashMap::new()).await.unwrap();
        let executor = DefaultQueryExecutor::new(Arc::new(memory_engine));
        
        let query = ParsedQuery::new(QueryType::Select, "SELECT * FROM users".to_string());
        let plan = OptimizedPlan::new(query);
        let context = ExecutionContext::new("test_query".to_string());
        
        let result = executor.execute(plan, context).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_query_cancellation() {
        let memory_engine = MemoryEngine::new(HashMap::new()).await.unwrap();
        let executor = DefaultQueryExecutor::new(Arc::new(memory_engine));
        
        let result = executor.cancel("test_query").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_executor_stats() {
        let memory_engine = MemoryEngine::new(HashMap::new()).await.unwrap();
        let executor = DefaultQueryExecutor::new(Arc::new(memory_engine));
        
        let stats = executor.get_stats().await.unwrap();
        assert!(stats.contains_key("running_queries"));
    }
}
