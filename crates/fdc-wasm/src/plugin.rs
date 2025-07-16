//! WASM plugin management

use crate::{
    runtime::WasmRuntime,
    types::WasmValue,
    security::SecurityPolicy,
};
use fdc_core::{
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

/// 插件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// 数据转换插件
    DataTransform,
    /// 自定义函数插件
    CustomFunction,
    /// 类型定义插件
    TypeDefinition,
    /// 索引优化插件
    IndexOptimizer,
    /// 压缩算法插件
    Compressor,
    /// 数据验证插件
    Validator,
    /// 聚合计算插件
    Aggregator,
    /// 序列化插件
    Serializer,
    /// 协议解析插件
    ProtocolParser,
    /// 自定义插件类型
    Custom(String),
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::DataTransform => write!(f, "data_transform"),
            PluginType::CustomFunction => write!(f, "custom_function"),
            PluginType::TypeDefinition => write!(f, "type_definition"),
            PluginType::IndexOptimizer => write!(f, "index_optimizer"),
            PluginType::Compressor => write!(f, "compressor"),
            PluginType::Validator => write!(f, "validator"),
            PluginType::Aggregator => write!(f, "aggregator"),
            PluginType::Serializer => write!(f, "serializer"),
            PluginType::ProtocolParser => write!(f, "protocol_parser"),
            PluginType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

/// 插件状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// 未加载
    NotLoaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 错误状态
    Error(String),
    /// 已卸载
    Unloaded,
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件ID
    pub id: Uuid,
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: Option<String>,
    /// 插件作者
    pub author: Option<String>,
    /// 插件类型
    pub plugin_type: PluginType,
    /// 插件状态
    pub status: PluginStatus,
    /// WASM文件路径
    pub wasm_path: PathBuf,
    /// 配置文件路径
    pub config_path: Option<PathBuf>,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: SystemTime,
    /// 依赖项
    pub dependencies: Vec<String>,
    /// 导出的函数
    pub exported_functions: Vec<String>,
    /// 权限要求
    pub permissions: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl PluginInfo {
    /// 创建新的插件信息
    pub fn new(
        name: String,
        version: String,
        plugin_type: PluginType,
        wasm_path: PathBuf,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            id: Uuid::new_v4(),
            name,
            version,
            description: None,
            author: None,
            plugin_type,
            status: PluginStatus::NotLoaded,
            wasm_path,
            config_path: None,
            created_at: now,
            updated_at: now,
            dependencies: Vec::new(),
            exported_functions: Vec::new(),
            permissions: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// 更新状态
    pub fn set_status(&mut self, status: PluginStatus) {
        self.status = status;
        self.updated_at = SystemTime::now();
    }
    
    /// 添加导出函数
    pub fn add_exported_function(&mut self, function_name: String) {
        if !self.exported_functions.contains(&function_name) {
            self.exported_functions.push(function_name);
            self.updated_at = SystemTime::now();
        }
    }
    
    /// 添加依赖
    pub fn add_dependency(&mut self, dependency: String) {
        if !self.dependencies.contains(&dependency) {
            self.dependencies.push(dependency);
            self.updated_at = SystemTime::now();
        }
    }
    
    /// 添加权限
    pub fn add_permission(&mut self, permission: String) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
            self.updated_at = SystemTime::now();
        }
    }
    
    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = SystemTime::now();
    }
    
    /// 检查是否可以运行
    pub fn can_run(&self) -> bool {
        matches!(self.status, PluginStatus::Loaded | PluginStatus::Paused)
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(self.status, PluginStatus::Running)
    }
    
    /// 检查是否有错误
    pub fn has_error(&self) -> bool {
        matches!(self.status, PluginStatus::Error(_))
    }
}

/// WASM插件实现
#[derive(Clone)]
pub struct WasmPlugin {
    /// 插件信息
    info: PluginInfo,
    /// WASM字节码
    wasm_bytes: Vec<u8>,
    /// 安全策略
    security_policy: SecurityPolicy,
    /// 配置参数
    config: HashMap<String, String>,
    /// 运行时统计
    stats: PluginStats,
}

/// 插件统计信息
#[derive(Debug, Clone, Default)]
pub struct PluginStats {
    /// 调用次数
    pub call_count: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub error_count: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub average_execution_time_ms: f64,
    /// 最后调用时间
    pub last_call_time: Option<SystemTime>,
    /// 最后错误
    pub last_error: Option<String>,
}

impl PluginStats {
    /// 记录函数调用
    pub fn record_call(&mut self, execution_time_ms: u64, success: bool, error: Option<String>) {
        self.call_count += 1;
        self.total_execution_time_ms += execution_time_ms;
        self.average_execution_time_ms = self.total_execution_time_ms as f64 / self.call_count as f64;
        self.last_call_time = Some(SystemTime::now());
        
        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
            self.last_error = error;
        }
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.call_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.call_count as f64
        }
    }
    
    /// 重置统计
    pub fn reset(&mut self) {
        *self = PluginStats::default();
    }
}

impl WasmPlugin {
    /// 创建新的WASM插件
    pub fn new(
        info: PluginInfo,
        wasm_bytes: Vec<u8>,
        security_policy: SecurityPolicy,
    ) -> Self {
        Self {
            info,
            wasm_bytes,
            security_policy,
            config: HashMap::new(),
            stats: PluginStats::default(),
        }
    }
    
    /// 获取插件信息
    pub fn info(&self) -> &PluginInfo {
        &self.info
    }
    
    /// 获取插件信息（可变）
    pub fn info_mut(&mut self) -> &mut PluginInfo {
        &mut self.info
    }
    
    /// 获取WASM字节码
    pub fn wasm_bytes(&self) -> &[u8] {
        &self.wasm_bytes
    }
    
    /// 获取安全策略
    pub fn security_policy(&self) -> &SecurityPolicy {
        &self.security_policy
    }
    
    /// 设置配置参数
    pub fn set_config(&mut self, key: String, value: String) {
        self.config.insert(key, value);
    }
    
    /// 获取配置参数
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }
    
    /// 获取所有配置
    pub fn config(&self) -> &HashMap<String, String> {
        &self.config
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> &PluginStats {
        &self.stats
    }
    
    /// 获取统计信息（可变）
    pub fn stats_mut(&mut self) -> &mut PluginStats {
        &mut self.stats
    }
    
    /// 调用插件函数
    pub fn call_function(
        &mut self,
        runtime: &WasmRuntime,
        function_name: &str,
        args: &[WasmValue],
    ) -> Result<WasmValue> {
        if !self.info.can_run() {
            return Err(Error::plugin(format!(
                "Plugin {} is not in a runnable state: {:?}",
                self.info.name, self.info.status
            )));
        }
        
        let start_time = std::time::Instant::now();
        
        // 更新状态为运行中
        self.info.set_status(PluginStatus::Running);
        
        // 调用运行时函数
        let result = runtime.call_function(&self.info.name, function_name, args);
        
        let execution_time = start_time.elapsed();
        let success = result.is_ok();
        let error = if let Err(ref e) = result {
            Some(e.to_string())
        } else {
            None
        };
        
        // 记录统计信息
        self.stats.record_call(execution_time.as_millis() as u64, success, error);
        
        // 更新状态
        if success {
            self.info.set_status(PluginStatus::Loaded);
        } else {
            self.info.set_status(PluginStatus::Error(
                result.as_ref().err().unwrap().to_string()
            ));
        }
        
        result
    }
    
    /// 验证插件
    pub fn validate(&self) -> Result<()> {
        // 检查WASM字节码
        if self.wasm_bytes.is_empty() {
            return Err(Error::validation("WASM bytes cannot be empty"));
        }
        
        // 检查插件名称
        if self.info.name.is_empty() {
            return Err(Error::validation("Plugin name cannot be empty"));
        }
        
        // 检查版本
        if self.info.version.is_empty() {
            return Err(Error::validation("Plugin version cannot be empty"));
        }
        
        // 检查WASM文件路径
        if !self.info.wasm_path.exists() {
            return Err(Error::validation(format!(
                "WASM file does not exist: {:?}",
                self.info.wasm_path
            )));
        }
        
        Ok(())
    }
    
    /// 重置插件状态
    pub fn reset(&mut self) {
        self.info.set_status(PluginStatus::NotLoaded);
        self.stats.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_plugin_info_creation() {
        let info = PluginInfo::new(
            "test_plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::DataTransform,
            PathBuf::from("/test/plugin.wasm"),
        );
        
        assert_eq!(info.name, "test_plugin");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.plugin_type, PluginType::DataTransform);
        assert_eq!(info.status, PluginStatus::NotLoaded);
        assert!(info.dependencies.is_empty());
        assert!(info.exported_functions.is_empty());
    }

    #[test]
    fn test_plugin_info_updates() {
        let mut info = PluginInfo::new(
            "test_plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::DataTransform,
            PathBuf::from("/test/plugin.wasm"),
        );
        
        info.add_exported_function("process_data".to_string());
        info.add_dependency("fdc-core".to_string());
        info.add_permission("read_data".to_string());
        info.set_metadata("author".to_string(), "test_author".to_string());
        
        assert_eq!(info.exported_functions.len(), 1);
        assert_eq!(info.dependencies.len(), 1);
        assert_eq!(info.permissions.len(), 1);
        assert_eq!(info.metadata.len(), 1);
    }

    #[test]
    fn test_plugin_stats() {
        let mut stats = PluginStats::default();
        
        stats.record_call(100, true, None);
        stats.record_call(200, false, Some("test error".to_string()));
        
        assert_eq!(stats.call_count, 2);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.success_rate(), 0.5);
        assert_eq!(stats.average_execution_time_ms, 150.0);
    }

    #[test]
    fn test_plugin_type_display() {
        assert_eq!(PluginType::DataTransform.to_string(), "data_transform");
        assert_eq!(PluginType::CustomFunction.to_string(), "custom_function");
        assert_eq!(PluginType::Custom("test".to_string()).to_string(), "custom_test");
    }
}
