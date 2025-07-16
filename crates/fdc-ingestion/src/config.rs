//! Ingestion configuration management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 数据接入配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    /// 接收器配置
    pub receiver: ReceiverConfig,
    /// 解析器配置
    pub parser: ParserConfig,
    /// 验证器配置
    pub validator: ValidatorConfig,
    /// 缓冲区配置
    pub buffer: BufferConfig,
    /// 批量处理配置
    pub batch: BatchConfig,
    /// 背压控制配置
    pub backpressure: BackpressureConfig,
    /// 恢复配置
    pub recovery: RecoveryConfig,
    /// 指标配置
    pub metrics: MetricsConfig,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            receiver: ReceiverConfig::default(),
            parser: ParserConfig::default(),
            validator: ValidatorConfig::default(),
            buffer: BufferConfig::default(),
            batch: BatchConfig::default(),
            backpressure: BackpressureConfig::default(),
            recovery: RecoveryConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

/// 接收器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiverConfig {
    /// 监听地址
    pub bind_address: String,
    /// 监听端口
    pub port: u16,
    /// 最大连接数
    pub max_connections: usize,
    /// 接收缓冲区大小
    pub receive_buffer_size: usize,
    /// 发送缓冲区大小
    pub send_buffer_size: usize,
    /// 连接超时时间
    pub connection_timeout: Duration,
    /// 读取超时时间
    pub read_timeout: Duration,
    /// 写入超时时间
    pub write_timeout: Duration,
    /// 是否启用TCP_NODELAY
    pub tcp_nodelay: bool,
    /// 是否启用SO_REUSEADDR
    pub reuse_addr: bool,
    /// 协议特定配置
    pub protocol_config: HashMap<String, String>,
}

impl Default for ReceiverConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            max_connections: crate::DEFAULT_MAX_CONNECTIONS,
            receive_buffer_size: crate::DEFAULT_RECEIVE_BUFFER_SIZE,
            send_buffer_size: 64 * 1024,
            connection_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(60),
            write_timeout: Duration::from_secs(30),
            tcp_nodelay: true,
            reuse_addr: true,
            protocol_config: HashMap::new(),
        }
    }
}

/// 解析器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// 解析器类型
    pub parser_type: String,
    /// 最大消息大小
    pub max_message_size: usize,
    /// 解析超时时间
    pub parse_timeout: Duration,
    /// 是否启用并行解析
    pub parallel_parsing: bool,
    /// 解析器线程数
    pub parser_threads: usize,
    /// 解析器特定配置
    pub parser_specific: HashMap<String, String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            parser_type: "json".to_string(),
            max_message_size: 1024 * 1024, // 1MB
            parse_timeout: Duration::from_millis(100),
            parallel_parsing: true,
            parser_threads: num_cpus::get(),
            parser_specific: HashMap::new(),
        }
    }
}

/// 验证器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// 是否启用验证
    pub enabled: bool,
    /// 验证规则
    pub rules: Vec<String>,
    /// 验证超时时间
    pub validation_timeout: Duration,
    /// 是否启用并行验证
    pub parallel_validation: bool,
    /// 验证器线程数
    pub validator_threads: usize,
    /// 严格模式
    pub strict_mode: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: vec!["required_fields".to_string(), "data_types".to_string()],
            validation_timeout: Duration::from_millis(50),
            parallel_validation: true,
            validator_threads: num_cpus::get() / 2,
            strict_mode: false,
        }
    }
}

/// 缓冲区配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    /// 缓冲区大小
    pub buffer_size: usize,
    /// 高水位标记
    pub high_watermark: f64,
    /// 低水位标记
    pub low_watermark: f64,
    /// 缓冲区类型
    pub buffer_type: String,
    /// 是否启用压缩
    pub compression_enabled: bool,
    /// 压缩算法
    pub compression_algorithm: String,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024 * 1024, // 1MB
            high_watermark: 0.8,
            low_watermark: 0.2,
            buffer_type: "ring".to_string(),
            compression_enabled: false,
            compression_algorithm: "lz4".to_string(),
        }
    }
}

/// 批量处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// 批量大小
    pub batch_size: usize,
    /// 批量超时时间
    pub batch_timeout: Duration,
    /// 最大批量大小
    pub max_batch_size: usize,
    /// 是否启用自适应批量
    pub adaptive_batching: bool,
    /// 批量处理线程数
    pub batch_threads: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: crate::DEFAULT_BATCH_SIZE,
            batch_timeout: Duration::from_millis(crate::DEFAULT_BATCH_TIMEOUT_MS),
            max_batch_size: 50000,
            adaptive_batching: true,
            batch_threads: num_cpus::get() / 4,
        }
    }
}

/// 背压控制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    /// 是否启用背压控制
    pub enabled: bool,
    /// 背压阈值
    pub threshold: f64,
    /// 背压策略
    pub strategy: String,
    /// 恢复阈值
    pub recovery_threshold: f64,
    /// 检查间隔
    pub check_interval: Duration,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 0.9,
            strategy: "drop_oldest".to_string(),
            recovery_threshold: 0.7,
            check_interval: Duration::from_millis(100),
        }
    }
}

/// 恢复配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// 是否启用自动恢复
    pub auto_recovery: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔
    pub retry_interval: Duration,
    /// 恢复策略
    pub recovery_strategy: String,
    /// 死信队列配置
    pub dead_letter_queue: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            auto_recovery: true,
            max_retries: 3,
            retry_interval: Duration::from_secs(1),
            recovery_strategy: "exponential_backoff".to_string(),
            dead_letter_queue: true,
        }
    }
}

/// 指标配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// 是否启用指标收集
    pub enabled: bool,
    /// 指标收集间隔
    pub collection_interval: Duration,
    /// 指标保留时间
    pub retention_period: Duration,
    /// 导出器配置
    pub exporters: Vec<String>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(10),
            retention_period: Duration::from_secs(24 * 60 * 60), // 24小时
            exporters: vec!["prometheus".to_string()],
        }
    }
}

impl IngestionConfig {
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
        if self.receiver.max_connections == 0 {
            return Err("max_connections must be greater than 0".to_string());
        }
        
        if self.buffer.high_watermark <= self.buffer.low_watermark {
            return Err("high_watermark must be greater than low_watermark".to_string());
        }
        
        if self.batch.batch_size == 0 {
            return Err("batch_size must be greater than 0".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IngestionConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.receiver.port, 8080);
        assert_eq!(config.batch.batch_size, crate::DEFAULT_BATCH_SIZE);
    }

    #[test]
    fn test_config_validation() {
        let mut config = IngestionConfig::default();
        
        // 测试无效的max_connections
        config.receiver.max_connections = 0;
        assert!(config.validate().is_err());
        
        // 恢复有效值
        config.receiver.max_connections = 100;
        assert!(config.validate().is_ok());
        
        // 测试无效的水位标记
        config.buffer.high_watermark = 0.5;
        config.buffer.low_watermark = 0.8;
        assert!(config.validate().is_err());
    }
}
