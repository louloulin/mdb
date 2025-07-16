//! Analytics configuration management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 分析引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// 流处理配置
    pub stream: StreamConfig,
    /// 批处理配置
    pub batch: BatchConfig,
    /// 机器学习配置
    pub ml: MLConfig,
    /// 风险计算配置
    pub risk: RiskConfig,
    /// 指标配置
    pub indicators: IndicatorsConfig,
    /// 聚合配置
    pub aggregation: AggregationConfig,
    /// 窗口配置
    pub windowing: WindowingConfig,
    /// 性能配置
    pub performance: PerformanceConfig,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            stream: StreamConfig::default(),
            batch: BatchConfig::default(),
            ml: MLConfig::default(),
            risk: RiskConfig::default(),
            indicators: IndicatorsConfig::default(),
            aggregation: AggregationConfig::default(),
            windowing: WindowingConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// 流处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// 缓冲区大小
    pub buffer_size: usize,
    /// 处理并行度
    pub parallelism: usize,
    /// 背压阈值
    pub backpressure_threshold: f64,
    /// 检查点间隔
    pub checkpoint_interval: Duration,
    /// 是否启用容错
    pub fault_tolerance: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            parallelism: crate::DEFAULT_PARALLELISM,
            backpressure_threshold: 0.8,
            checkpoint_interval: Duration::from_secs(30),
            fault_tolerance: true,
        }
    }
}

/// 批处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// 批处理大小
    pub batch_size: usize,
    /// 最大批处理大小
    pub max_batch_size: usize,
    /// 批处理超时
    pub batch_timeout: Duration,
    /// 并行批次数
    pub parallel_batches: usize,
    /// 内存限制（字节）
    pub memory_limit: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: crate::DEFAULT_BATCH_SIZE,
            max_batch_size: 100000,
            batch_timeout: Duration::from_secs(60),
            parallel_batches: 4,
            memory_limit: 1024 * 1024 * 1024, // 1GB
        }
    }
}

/// 机器学习配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    /// 模型类型
    pub model_type: String,
    /// 训练参数
    pub training_params: HashMap<String, f64>,
    /// 预测窗口大小
    pub prediction_window: usize,
    /// 特征数量
    pub feature_count: usize,
    /// 学习率
    pub learning_rate: f64,
    /// 批次大小
    pub batch_size: usize,
    /// 训练轮数
    pub epochs: usize,
}

impl Default for MLConfig {
    fn default() -> Self {
        let mut training_params = HashMap::new();
        training_params.insert("regularization".to_string(), 0.01);
        training_params.insert("dropout".to_string(), 0.1);
        
        Self {
            model_type: "lstm".to_string(),
            training_params,
            prediction_window: 100,
            feature_count: 10,
            learning_rate: 0.001,
            batch_size: 32,
            epochs: 100,
        }
    }
}

/// 风险计算配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// VaR置信水平
    pub var_confidence_level: f64,
    /// 历史数据窗口大小
    pub historical_window: usize,
    /// 蒙特卡洛模拟次数
    pub monte_carlo_simulations: usize,
    /// 风险因子
    pub risk_factors: Vec<String>,
    /// 相关性计算窗口
    pub correlation_window: usize,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            var_confidence_level: 0.95,
            historical_window: 252, // 一年交易日
            monte_carlo_simulations: 10000,
            risk_factors: vec![
                "market_risk".to_string(),
                "credit_risk".to_string(),
                "liquidity_risk".to_string(),
            ],
            correlation_window: 60,
        }
    }
}

/// 技术指标配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorsConfig {
    /// 移动平均窗口大小
    pub ma_windows: Vec<usize>,
    /// RSI周期
    pub rsi_period: usize,
    /// MACD参数
    pub macd_params: (usize, usize, usize), // (fast, slow, signal)
    /// 布林带参数
    pub bollinger_params: (usize, f64), // (period, std_dev)
    /// 是否启用所有指标
    pub enable_all: bool,
}

impl Default for IndicatorsConfig {
    fn default() -> Self {
        Self {
            ma_windows: vec![5, 10, 20, 50, 200],
            rsi_period: 14,
            macd_params: (12, 26, 9),
            bollinger_params: (20, 2.0),
            enable_all: true,
        }
    }
}

/// 聚合配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// 聚合函数
    pub functions: Vec<String>,
    /// 分组字段
    pub group_by_fields: Vec<String>,
    /// 聚合窗口
    pub window_size: Duration,
    /// 是否启用增量聚合
    pub incremental: bool,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            functions: vec![
                "sum".to_string(),
                "avg".to_string(),
                "min".to_string(),
                "max".to_string(),
                "count".to_string(),
            ],
            group_by_fields: vec!["symbol".to_string()],
            window_size: Duration::from_secs(crate::DEFAULT_WINDOW_SIZE_SECS),
            incremental: true,
        }
    }
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowingConfig {
    /// 默认窗口大小
    pub default_window_size: Duration,
    /// 滑动窗口步长
    pub slide_interval: Duration,
    /// 最大窗口数量
    pub max_windows: usize,
    /// 窗口类型
    pub window_type: String,
}

impl Default for WindowingConfig {
    fn default() -> Self {
        Self {
            default_window_size: Duration::from_secs(crate::DEFAULT_WINDOW_SIZE_SECS),
            slide_interval: Duration::from_secs(10),
            max_windows: 100,
            window_type: "tumbling".to_string(),
        }
    }
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 工作线程数
    pub worker_threads: usize,
    /// 最大内存使用量（字节）
    pub max_memory_usage: usize,
    /// CPU使用率限制
    pub cpu_limit: f64,
    /// 是否启用性能监控
    pub monitoring_enabled: bool,
    /// 性能报告间隔
    pub report_interval: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            max_memory_usage: 2 * 1024 * 1024 * 1024, // 2GB
            cpu_limit: 0.8,
            monitoring_enabled: true,
            report_interval: Duration::from_secs(60),
        }
    }
}

impl AnalyticsConfig {
    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.stream.buffer_size == 0 {
            return Err("stream buffer_size must be greater than 0".to_string());
        }
        
        if self.batch.batch_size == 0 {
            return Err("batch batch_size must be greater than 0".to_string());
        }
        
        if self.ml.learning_rate <= 0.0 || self.ml.learning_rate >= 1.0 {
            return Err("ml learning_rate must be between 0 and 1".to_string());
        }
        
        if self.risk.var_confidence_level <= 0.0 || self.risk.var_confidence_level >= 1.0 {
            return Err("risk var_confidence_level must be between 0 and 1".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnalyticsConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.stream.parallelism, crate::DEFAULT_PARALLELISM);
        assert_eq!(config.batch.batch_size, crate::DEFAULT_BATCH_SIZE);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AnalyticsConfig::default();
        
        // 测试无效的buffer_size
        config.stream.buffer_size = 0;
        assert!(config.validate().is_err());
        
        // 恢复有效值
        config.stream.buffer_size = 1000;
        assert!(config.validate().is_ok());
        
        // 测试无效的学习率
        config.ml.learning_rate = 1.5;
        assert!(config.validate().is_err());
    }
}
