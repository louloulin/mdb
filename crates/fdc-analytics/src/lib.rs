//! # Financial Data Center Analytics Engine
//!
//! This crate provides comprehensive analytics capabilities for the Financial Data Center,
//! including stream processing, batch processing, machine learning, and risk calculations.

pub mod stream;         // 流处理引擎
pub mod batch;          // 批处理引擎
pub mod ml;             // 机器学习引擎
pub mod risk;           // 风险计算引擎
pub mod indicators;     // 技术指标计算
pub mod aggregation;    // 数据聚合
pub mod windowing;      // 时间窗口处理
pub mod pipeline;       // 分析管道
pub mod config;         // 配置管理
pub mod metrics;        // 分析指标
pub mod models;         // 数据模型

// 重新导出常用类型
pub use stream::StreamProcessor;
pub use batch::BatchProcessor;
pub use ml::MLEngine;
pub use risk::RiskEngine;
pub use indicators::TechnicalIndicators;
pub use config::{
    AnalyticsConfig, StreamConfig, BatchConfig, MLConfig, RiskConfig,
    IndicatorsConfig, AggregationConfig, WindowingConfig, PerformanceConfig
};
pub use models::{AnalyticsResult, TimeSeriesData, MarketData};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 默认批处理大小
pub const DEFAULT_BATCH_SIZE: usize = 10000;

/// 默认窗口大小（秒）
pub const DEFAULT_WINDOW_SIZE_SECS: u64 = 60;

/// 默认并行度
pub const DEFAULT_PARALLELISM: usize = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "fdc-analytics");
        assert_eq!(DEFAULT_BATCH_SIZE, 10000);
        assert_eq!(DEFAULT_WINDOW_SIZE_SECS, 60);
        assert_eq!(DEFAULT_PARALLELISM, 4);
    }
}
