//! # Financial Data Center High-Performance Query Engine
//!
//! This crate provides a comprehensive query engine for the Financial Data Center,
//! featuring SQL parsing, query optimization, execution planning, and caching.

pub mod parser;         // SQL解析器
pub mod optimizer;      // 查询优化器
pub mod executor;       // 查询执行器
pub mod planner;        // 查询计划器
pub mod cache;          // 查询缓存
pub mod engine;         // 查询引擎
pub mod functions;      // 内置函数
pub mod aggregates;     // 聚合函数
pub mod joins;          // 连接操作
pub mod filters;        // 过滤器
pub mod projections;    // 投影操作
pub mod sorts;          // 排序操作
pub mod metrics;        // 查询指标
pub mod config;         // 配置管理

// 重新导出常用类型
pub use engine::{QueryEngine, QueryEngineConfig};
pub use parser::{SqlParser, ParsedQuery, QueryType};
pub use optimizer::{QueryOptimizer, OptimizationRule, OptimizedPlan};
pub use executor::{QueryExecutor, ExecutionContext, ExecutionResult};
pub use planner::{QueryPlanner, ExecutionPlan, PlanNode};
pub use cache::{QueryCache, CachePolicy, CacheStats};
pub use functions::BuiltinFunctions;
pub use aggregates::AggregateFunction;
pub use metrics::QueryMetrics;
pub use config::QueryConfig;

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 默认查询缓存大小 (100MB)
pub const DEFAULT_CACHE_SIZE: usize = 100 * 1024 * 1024;

/// 默认查询超时时间 (30秒)
pub const DEFAULT_QUERY_TIMEOUT_SECS: u64 = 30;

/// 默认最大并发查询数
pub const DEFAULT_MAX_CONCURRENT_QUERIES: usize = 100;

/// 默认结果集大小限制 (10MB)
pub const DEFAULT_MAX_RESULT_SIZE: usize = 10 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "fdc-query");
        assert_eq!(DEFAULT_CACHE_SIZE, 100 * 1024 * 1024);
        assert_eq!(DEFAULT_QUERY_TIMEOUT_SECS, 30);
        assert_eq!(DEFAULT_MAX_CONCURRENT_QUERIES, 100);
        assert_eq!(DEFAULT_MAX_RESULT_SIZE, 10 * 1024 * 1024);
    }
}
