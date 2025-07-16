//! WASM bridge utilities for Financial Data Center

use crate::error::{Error, Result};
use crate::types::{Value, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WASM函数调用接口
pub trait WasmFunction {
    /// 调用WASM函数
    fn call(&self, args: &[Value]) -> Result<Value>;
    
    /// 获取函数名称
    fn name(&self) -> &str;
    
    /// 获取函数签名
    fn signature(&self) -> &FunctionSignature;
}

/// 函数签名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub input_types: Vec<TypeId>,
    pub output_type: TypeId,
    pub description: Option<String>,
}

/// WASM模块接口
pub trait WasmModule {
    /// 获取模块名称
    fn name(&self) -> &str;
    
    /// 获取模块版本
    fn version(&self) -> &str;
    
    /// 获取所有函数
    fn functions(&self) -> &HashMap<String, Box<dyn WasmFunction>>;
    
    /// 调用函数
    fn call_function(&self, name: &str, args: &[Value]) -> Result<Value>;
    
    /// 检查函数是否存在
    fn has_function(&self, name: &str) -> bool;
    
    /// 初始化模块
    fn initialize(&mut self) -> Result<()>;
    
    /// 清理模块
    fn cleanup(&mut self) -> Result<()>;
}

/// WASM运行时接口
pub trait WasmRuntime {
    /// 加载模块
    fn load_module(&mut self, module_bytes: &[u8]) -> Result<Box<dyn WasmModule>>;
    
    /// 卸载模块
    fn unload_module(&mut self, module_name: &str) -> Result<()>;
    
    /// 获取模块
    fn get_module(&self, module_name: &str) -> Option<&dyn WasmModule>;
    
    /// 列出所有模块
    fn list_modules(&self) -> Vec<String>;
    
    /// 设置内存限制
    fn set_memory_limit(&mut self, limit_bytes: usize);
    
    /// 设置执行超时
    fn set_execution_timeout(&mut self, timeout_ms: u64);
}

/// WASM值转换器
pub struct WasmValueConverter;

impl WasmValueConverter {
    /// 将Value转换为WASM兼容的字节数组
    pub fn value_to_bytes(value: &Value) -> Result<Vec<u8>> {
        bincode::serialize(value).map_err(|e| Error::serialization(e.to_string()))
    }
    
    /// 将字节数组转换为Value
    pub fn bytes_to_value(bytes: &[u8]) -> Result<Value> {
        bincode::deserialize(bytes).map_err(|e| Error::serialization(e.to_string()))
    }
    
    /// 将Value转换为JSON字符串
    pub fn value_to_json(value: &Value) -> Result<String> {
        serde_json::to_string(value).map_err(|e| Error::serialization(e.to_string()))
    }
    
    /// 将JSON字符串转换为Value
    pub fn json_to_value(json: &str) -> Result<Value> {
        serde_json::from_str(json).map_err(|e| Error::serialization(e.to_string()))
    }
}

/// WASM安全策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSecurityPolicy {
    /// 内存限制（字节）
    pub memory_limit: usize,
    /// 执行超时（毫秒）
    pub execution_timeout: u64,
    /// 允许的系统调用
    pub allowed_syscalls: Vec<String>,
    /// 是否允许网络访问
    pub network_access: bool,
    /// 是否允许文件访问
    pub file_access: bool,
    /// 是否允许环境变量访问
    pub env_access: bool,
}

impl Default for WasmSecurityPolicy {
    fn default() -> Self {
        Self {
            memory_limit: 128 * 1024 * 1024, // 128MB
            execution_timeout: 5000,          // 5秒
            allowed_syscalls: vec![],
            network_access: false,
            file_access: false,
            env_access: false,
        }
    }
}

/// WASM执行上下文
#[derive(Debug)]
pub struct WasmExecutionContext {
    /// 模块名称
    pub module_name: String,
    /// 函数名称
    pub function_name: String,
    /// 输入参数
    pub input_args: Vec<Value>,
    /// 执行开始时间
    pub start_time: std::time::Instant,
    /// 安全策略
    pub security_policy: WasmSecurityPolicy,
}

impl WasmExecutionContext {
    /// 创建新的执行上下文
    pub fn new(
        module_name: String,
        function_name: String,
        input_args: Vec<Value>,
        security_policy: WasmSecurityPolicy,
    ) -> Self {
        Self {
            module_name,
            function_name,
            input_args,
            start_time: std::time::Instant::now(),
            security_policy,
        }
    }
    
    /// 检查是否超时
    pub fn is_timeout(&self) -> bool {
        self.start_time.elapsed().as_millis() > self.security_policy.execution_timeout as u128
    }
    
    /// 获取执行时间
    pub fn execution_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// WASM执行结果
#[derive(Debug)]
pub struct WasmExecutionResult {
    /// 返回值
    pub return_value: Value,
    /// 执行时间
    pub execution_time: std::time::Duration,
    /// 内存使用量
    pub memory_usage: usize,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error_message: Option<String>,
}

/// WASM事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmEvent {
    /// 模块加载
    ModuleLoaded {
        module_name: String,
        module_size: usize,
    },
    /// 模块卸载
    ModuleUnloaded {
        module_name: String,
    },
    /// 函数调用开始
    FunctionCallStarted {
        module_name: String,
        function_name: String,
        args_count: usize,
    },
    /// 函数调用完成
    FunctionCallCompleted {
        module_name: String,
        function_name: String,
        execution_time_ms: u64,
        success: bool,
    },
    /// 内存限制超出
    MemoryLimitExceeded {
        module_name: String,
        current_usage: usize,
        limit: usize,
    },
    /// 执行超时
    ExecutionTimeout {
        module_name: String,
        function_name: String,
        timeout_ms: u64,
    },
    /// 安全违规
    SecurityViolation {
        module_name: String,
        violation_type: String,
        details: String,
    },
}

/// WASM事件监听器
pub trait WasmEventListener {
    /// 处理WASM事件
    fn on_event(&self, event: WasmEvent);
}

/// WASM指标收集器
#[derive(Debug, Default)]
pub struct WasmMetrics {
    /// 模块加载次数
    pub modules_loaded: u64,
    /// 模块卸载次数
    pub modules_unloaded: u64,
    /// 函数调用次数
    pub function_calls: u64,
    /// 成功调用次数
    pub successful_calls: u64,
    /// 失败调用次数
    pub failed_calls: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub average_execution_time_ms: f64,
    /// 内存使用峰值
    pub peak_memory_usage: usize,
    /// 超时次数
    pub timeout_count: u64,
    /// 安全违规次数
    pub security_violations: u64,
}

impl WasmMetrics {
    /// 记录函数调用
    pub fn record_function_call(&mut self, execution_time_ms: u64, success: bool, memory_usage: usize) {
        self.function_calls += 1;
        if success {
            self.successful_calls += 1;
        } else {
            self.failed_calls += 1;
        }
        
        self.total_execution_time_ms += execution_time_ms;
        self.average_execution_time_ms = self.total_execution_time_ms as f64 / self.function_calls as f64;
        
        if memory_usage > self.peak_memory_usage {
            self.peak_memory_usage = memory_usage;
        }
    }
    
    /// 记录模块加载
    pub fn record_module_loaded(&mut self) {
        self.modules_loaded += 1;
    }
    
    /// 记录模块卸载
    pub fn record_module_unloaded(&mut self) {
        self.modules_unloaded += 1;
    }
    
    /// 记录超时
    pub fn record_timeout(&mut self) {
        self.timeout_count += 1;
        self.failed_calls += 1;
    }
    
    /// 记录安全违规
    pub fn record_security_violation(&mut self) {
        self.security_violations += 1;
        self.failed_calls += 1;
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.function_calls == 0 {
            0.0
        } else {
            self.successful_calls as f64 / self.function_calls as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_wasm_value_converter() {
        let value = Value::Int32(42);
        let bytes = WasmValueConverter::value_to_bytes(&value).unwrap();
        let converted_back = WasmValueConverter::bytes_to_value(&bytes).unwrap();
        
        match converted_back {
            Value::Int32(n) => assert_eq!(n, 42),
            _ => panic!("Unexpected value type"),
        }
    }

    #[test]
    fn test_wasm_security_policy() {
        let policy = WasmSecurityPolicy::default();
        assert_eq!(policy.memory_limit, 128 * 1024 * 1024);
        assert_eq!(policy.execution_timeout, 5000);
        assert!(!policy.network_access);
        assert!(!policy.file_access);
    }

    #[test]
    fn test_wasm_execution_context() {
        let policy = WasmSecurityPolicy::default();
        let context = WasmExecutionContext::new(
            "test_module".to_string(),
            "test_function".to_string(),
            vec![Value::Int32(42)],
            policy,
        );
        
        assert_eq!(context.module_name, "test_module");
        assert_eq!(context.function_name, "test_function");
        assert_eq!(context.input_args.len(), 1);
        assert!(!context.is_timeout()); // 应该不会立即超时
    }

    #[test]
    fn test_wasm_metrics() {
        let mut metrics = WasmMetrics::default();
        
        metrics.record_function_call(100, true, 1024);
        metrics.record_function_call(200, false, 2048);
        
        assert_eq!(metrics.function_calls, 2);
        assert_eq!(metrics.successful_calls, 1);
        assert_eq!(metrics.failed_calls, 1);
        assert_eq!(metrics.success_rate(), 0.5);
        assert_eq!(metrics.average_execution_time_ms, 150.0);
        assert_eq!(metrics.peak_memory_usage, 2048);
    }
}
