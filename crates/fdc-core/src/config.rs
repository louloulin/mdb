//! Configuration management for Financial Data Center

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::error::{Error, Result};

/// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub ingestion: IngestionConfig,
    pub query: QueryConfig,
    pub monitoring: MonitoringConfig,
    pub wasm: WasmConfig,
    pub types: TypesConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            ingestion: IngestionConfig::default(),
            query: QueryConfig::default(),
            monitoring: MonitoringConfig::default(),
            wasm: WasmConfig::default(),
            types: TypesConfig::default(),
        }
    }
}

impl Config {
    /// 从文件加载配置
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| Error::config(format!("Failed to read config file {:?}: {}", path, e)))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| Error::config(format!("Failed to parse config file {:?}: {}", path, e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();
        
        // 服务器配置
        if let Ok(host) = std::env::var("FDC_SERVER_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("FDC_SERVER_REST_PORT") {
            config.server.rest_port = port.parse()
                .map_err(|e| Error::config(format!("Invalid REST port: {}", e)))?;
        }
        
        // 存储配置
        if let Ok(path) = std::env::var("FDC_STORAGE_REALTIME_PATH") {
            config.storage.realtime.path = PathBuf::from(path);
        }
        
        config.validate()?;
        Ok(config)
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        self.server.validate()?;
        self.storage.validate()?;
        self.ingestion.validate()?;
        self.query.validate()?;
        self.monitoring.validate()?;
        self.wasm.validate()?;
        self.types.validate()?;
        Ok(())
    }
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub rest_port: u16,
    pub grpc_port: u16,
    pub graphql_port: u16,
    pub websocket_port: u16,
    pub metrics_port: u16,
    pub admin_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            rest_port: 8080,
            grpc_port: 9090,
            graphql_port: 8081,
            websocket_port: 8082,
            metrics_port: 9091,
            admin_port: 8083,
        }
    }
}

impl ServerConfig {
    fn validate(&self) -> Result<()> {
        if self.host.is_empty() {
            return Err(Error::config("Server host cannot be empty"));
        }
        
        let ports = [
            self.rest_port,
            self.grpc_port,
            self.graphql_port,
            self.websocket_port,
            self.metrics_port,
            self.admin_port,
        ];
        
        for (i, &port1) in ports.iter().enumerate() {
            for &port2 in ports.iter().skip(i + 1) {
                if port1 == port2 {
                    return Err(Error::config(format!("Duplicate port: {}", port1)));
                }
            }
        }
        
        Ok(())
    }
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub realtime: RealtimeStorageConfig,
    pub analytical: AnalyticalStorageConfig,
    pub archive: ArchiveStorageConfig,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            realtime: RealtimeStorageConfig::default(),
            analytical: AnalyticalStorageConfig::default(),
            archive: ArchiveStorageConfig::default(),
        }
    }
}

impl StorageConfig {
    fn validate(&self) -> Result<()> {
        self.realtime.validate()?;
        self.analytical.validate()?;
        self.archive.validate()?;
        Ok(())
    }
}

/// 实时存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeStorageConfig {
    pub engine: String,
    pub path: PathBuf,
    pub memory_limit: String,
    pub sync_interval: String,
}

impl Default for RealtimeStorageConfig {
    fn default() -> Self {
        Self {
            engine: "redb".to_string(),
            path: PathBuf::from("/data/realtime"),
            memory_limit: "16GB".to_string(),
            sync_interval: "1s".to_string(),
        }
    }
}

impl RealtimeStorageConfig {
    fn validate(&self) -> Result<()> {
        if !["redb", "memory"].contains(&self.engine.as_str()) {
            return Err(Error::config(format!("Invalid realtime engine: {}", self.engine)));
        }
        Ok(())
    }
}

/// 分析存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticalStorageConfig {
    pub engine: String,
    pub path: PathBuf,
    pub memory_limit: String,
    pub threads: u32,
}

impl Default for AnalyticalStorageConfig {
    fn default() -> Self {
        Self {
            engine: "duckdb".to_string(),
            path: PathBuf::from("/data/analytical"),
            memory_limit: "64GB".to_string(),
            threads: 32,
        }
    }
}

impl AnalyticalStorageConfig {
    fn validate(&self) -> Result<()> {
        if !["duckdb", "datafusion"].contains(&self.engine.as_str()) {
            return Err(Error::config(format!("Invalid analytical engine: {}", self.engine)));
        }
        if self.threads == 0 {
            return Err(Error::config("Threads must be greater than 0"));
        }
        Ok(())
    }
}

/// 归档存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStorageConfig {
    pub engine: String,
    pub path: PathBuf,
    pub compression: String,
    pub block_cache_size: String,
}

impl Default for ArchiveStorageConfig {
    fn default() -> Self {
        Self {
            engine: "rocksdb".to_string(),
            path: PathBuf::from("/data/archive"),
            compression: "lz4".to_string(),
            block_cache_size: "8GB".to_string(),
        }
    }
}

impl ArchiveStorageConfig {
    fn validate(&self) -> Result<()> {
        if !["rocksdb", "sled"].contains(&self.engine.as_str()) {
            return Err(Error::config(format!("Invalid archive engine: {}", self.engine)));
        }
        if !["lz4", "zstd", "snappy", "none"].contains(&self.compression.as_str()) {
            return Err(Error::config(format!("Invalid compression: {}", self.compression)));
        }
        Ok(())
    }
}

/// 数据接入配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    pub sources: Vec<DataSourceConfig>,
    pub buffer_size: String,
    pub batch_size: u32,
    pub flush_interval: String,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            sources: vec![],
            buffer_size: "1MB".to_string(),
            batch_size: 1000,
            flush_interval: "100ms".to_string(),
        }
    }
}

impl IngestionConfig {
    fn validate(&self) -> Result<()> {
        if self.batch_size == 0 {
            return Err(Error::config("Batch size must be greater than 0"));
        }
        for source in &self.sources {
            source.validate()?;
        }
        Ok(())
    }
}

/// 数据源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    pub name: String,
    pub source_type: String,
    pub address: String,
    pub protocol: String,
    pub enabled: bool,
    pub properties: HashMap<String, String>,
}

impl DataSourceConfig {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::config("Data source name cannot be empty"));
        }
        if !["multicast", "tcp", "websocket", "kafka"].contains(&self.source_type.as_str()) {
            return Err(Error::config(format!("Invalid source type: {}", self.source_type)));
        }
        Ok(())
    }
}

/// 查询配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    pub cache_size: String,
    pub cache_ttl: String,
    pub max_concurrent_queries: u32,
    pub query_timeout: String,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            cache_size: "4GB".to_string(),
            cache_ttl: "300s".to_string(),
            max_concurrent_queries: 1000,
            query_timeout: "30s".to_string(),
        }
    }
}

impl QueryConfig {
    fn validate(&self) -> Result<()> {
        if self.max_concurrent_queries == 0 {
            return Err(Error::config("Max concurrent queries must be greater than 0"));
        }
        Ok(())
    }
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_port: u16,
    pub log_level: String,
    pub trace_sampling_rate: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_port: 9091,
            log_level: "info".to_string(),
            trace_sampling_rate: 0.1,
        }
    }
}

impl MonitoringConfig {
    fn validate(&self) -> Result<()> {
        if !["trace", "debug", "info", "warn", "error"].contains(&self.log_level.as_str()) {
            return Err(Error::config(format!("Invalid log level: {}", self.log_level)));
        }
        if !(0.0..=1.0).contains(&self.trace_sampling_rate) {
            return Err(Error::config("Trace sampling rate must be between 0.0 and 1.0"));
        }
        Ok(())
    }
}

/// WASM配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    pub enabled: bool,
    pub plugin_dir: PathBuf,
    pub memory_limit: String,
    pub execution_timeout: String,
    pub max_plugins: u32,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            plugin_dir: PathBuf::from("./plugins"),
            memory_limit: "128MB".to_string(),
            execution_timeout: "5s".to_string(),
            max_plugins: 100,
        }
    }
}

impl WasmConfig {
    fn validate(&self) -> Result<()> {
        if self.max_plugins == 0 {
            return Err(Error::config("Max plugins must be greater than 0"));
        }
        Ok(())
    }
}

/// 类型系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypesConfig {
    pub schema_dir: PathBuf,
    pub auto_register: bool,
    pub validation_enabled: bool,
}

impl Default for TypesConfig {
    fn default() -> Self {
        Self {
            schema_dir: PathBuf::from("./schemas"),
            auto_register: true,
            validation_enabled: true,
        }
    }
}

impl TypesConfig {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();
        config.rest_port = config.grpc_port; // 重复端口
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_storage_config_validation() {
        let mut config = RealtimeStorageConfig::default();
        config.engine = "invalid".to_string();
        assert!(config.validate().is_err());
    }
}
