//! WASM-Host bridge implementation

use crate::types::{WasmValue, WasmType, WasmFunctionSignature};
use fdc_core::{
    error::{Error, Result},
    types::Value,
};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// 主机函数特征
pub trait HostFunction: Send + Sync {
    /// 函数名称
    fn name(&self) -> &str;
    
    /// 函数签名
    fn signature(&self) -> &WasmFunctionSignature;
    
    /// 调用函数
    fn call(&self, args: &[WasmValue]) -> Result<WasmValue>;
    
    /// 函数描述
    fn description(&self) -> Option<&str> {
        None
    }
    
    /// 检查是否为异步函数
    fn is_async(&self) -> bool {
        false
    }
}

/// 简单主机函数实现
pub struct SimpleHostFunction {
    name: String,
    signature: WasmFunctionSignature,
    description: Option<String>,
    handler: Box<dyn Fn(&[WasmValue]) -> Result<WasmValue> + Send + Sync>,
}

impl SimpleHostFunction {
    /// 创建新的简单主机函数
    pub fn new<F>(
        name: String,
        signature: WasmFunctionSignature,
        handler: F,
    ) -> Self
    where
        F: Fn(&[WasmValue]) -> Result<WasmValue> + Send + Sync + 'static,
    {
        Self {
            name,
            signature,
            description: None,
            handler: Box::new(handler),
        }
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl HostFunction for SimpleHostFunction {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn signature(&self) -> &WasmFunctionSignature {
        &self.signature
    }
    
    fn call(&self, args: &[WasmValue]) -> Result<WasmValue> {
        // 验证参数
        self.signature.validate_args(args)?;
        
        // 调用处理函数
        let result = (self.handler)(args)?;
        
        // 验证返回值
        self.signature.validate_return(&result)?;
        
        Ok(result)
    }
    
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// WASM桥接器
pub struct WasmBridge {
    /// 注册的主机函数
    host_functions: Arc<RwLock<HashMap<String, Box<dyn HostFunction>>>>,
    /// 函数调用统计
    call_stats: Arc<RwLock<HashMap<String, FunctionCallStats>>>,
}

/// 函数调用统计
#[derive(Debug, Clone, Default)]
pub struct FunctionCallStats {
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
    pub last_call_time: Option<std::time::SystemTime>,
    /// 最后错误
    pub last_error: Option<String>,
}

impl FunctionCallStats {
    /// 记录函数调用
    pub fn record_call(&mut self, execution_time_ms: u64, success: bool, error: Option<String>) {
        self.call_count += 1;
        self.total_execution_time_ms += execution_time_ms;
        self.average_execution_time_ms = self.total_execution_time_ms as f64 / self.call_count as f64;
        self.last_call_time = Some(std::time::SystemTime::now());
        
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
}

impl WasmBridge {
    /// 创建新的WASM桥接器
    pub fn new() -> Self {
        Self {
            host_functions: Arc::new(RwLock::new(HashMap::new())),
            call_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册主机函数
    pub fn register_host_function(&self, function: Box<dyn HostFunction>) -> Result<()> {
        let name = function.name().to_string();
        
        // 检查函数是否已存在
        {
            let functions = self.host_functions.read();
            if functions.contains_key(&name) {
                return Err(Error::already_exists(format!("Host function: {}", name)));
            }
        }
        
        // 注册函数
        {
            let mut functions = self.host_functions.write();
            functions.insert(name.clone(), function);
        }
        
        // 初始化统计
        {
            let mut stats = self.call_stats.write();
            stats.insert(name, FunctionCallStats::default());
        }
        
        Ok(())
    }
    
    /// 取消注册主机函数
    pub fn unregister_host_function(&self, name: &str) -> Result<()> {
        let mut functions = self.host_functions.write();
        let mut stats = self.call_stats.write();
        
        if functions.remove(name).is_none() {
            return Err(Error::not_found(format!("Host function: {}", name)));
        }
        
        stats.remove(name);
        Ok(())
    }
    
    /// 调用主机函数
    pub fn call_host_function(&self, name: &str, args: &[WasmValue]) -> Result<WasmValue> {
        let start_time = std::time::Instant::now();
        
        // 获取函数
        let function = {
            let functions = self.host_functions.read();
            functions.get(name)
                .ok_or_else(|| Error::not_found(format!("Host function: {}", name)))?
                .as_ref() as *const dyn HostFunction
        };
        
        // 调用函数（需要unsafe因为我们绕过了借用检查器）
        let result = unsafe { (*function).call(args) };
        
        let execution_time = start_time.elapsed();
        let success = result.is_ok();
        let error = if let Err(ref e) = result {
            Some(e.to_string())
        } else {
            None
        };
        
        // 更新统计
        {
            let mut stats = self.call_stats.write();
            if let Some(function_stats) = stats.get_mut(name) {
                function_stats.record_call(execution_time.as_millis() as u64, success, error);
            }
        }
        
        result
    }
    
    /// 列出所有主机函数
    pub fn list_host_functions(&self) -> Vec<String> {
        self.host_functions.read().keys().cloned().collect()
    }
    
    /// 获取函数签名
    pub fn get_function_signature(&self, name: &str) -> Option<WasmFunctionSignature> {
        self.host_functions.read()
            .get(name)
            .map(|f| f.signature().clone())
    }
    
    /// 获取函数描述
    pub fn get_function_description(&self, name: &str) -> Option<String> {
        self.host_functions.read()
            .get(name)
            .and_then(|f| f.description().map(|s| s.to_string()))
    }
    
    /// 获取函数统计
    pub fn get_function_stats(&self, name: &str) -> Option<FunctionCallStats> {
        self.call_stats.read().get(name).cloned()
    }
    
    /// 获取所有函数统计
    pub fn get_all_function_stats(&self) -> HashMap<String, FunctionCallStats> {
        self.call_stats.read().clone()
    }
    
    /// 重置函数统计
    pub fn reset_function_stats(&self, name: &str) -> Result<()> {
        let mut stats = self.call_stats.write();
        if let Some(function_stats) = stats.get_mut(name) {
            *function_stats = FunctionCallStats::default();
            Ok(())
        } else {
            Err(Error::not_found(format!("Host function: {}", name)))
        }
    }
    
    /// 重置所有统计
    pub fn reset_all_stats(&self) {
        let mut stats = self.call_stats.write();
        for function_stats in stats.values_mut() {
            *function_stats = FunctionCallStats::default();
        }
    }
    
    /// 检查函数是否存在
    pub fn has_function(&self, name: &str) -> bool {
        self.host_functions.read().contains_key(name)
    }
    
    /// 获取函数数量
    pub fn function_count(&self) -> usize {
        self.host_functions.read().len()
    }
}

impl Default for WasmBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建标准主机函数
pub fn create_standard_host_functions() -> Vec<Box<dyn HostFunction>> {
    let mut functions: Vec<Box<dyn HostFunction>> = Vec::new();
    
    // 日志函数
    let log_function = SimpleHostFunction::new(
        "log".to_string(),
        WasmFunctionSignature::new(
            "log".to_string(),
            vec![WasmType::String],
            WasmType::Null,
        ).with_description("Log a message to the console".to_string()),
        |args| {
            if let Some(WasmValue::String(message)) = args.get(0) {
                println!("[WASM LOG] {}", message);
            }
            Ok(WasmValue::Null)
        },
    ).with_description("Log a message to the console".to_string());
    
    functions.push(Box::new(log_function));
    
    // 时间戳函数
    let timestamp_function = SimpleHostFunction::new(
        "timestamp".to_string(),
        WasmFunctionSignature::new(
            "timestamp".to_string(),
            vec![],
            WasmType::I64,
        ).with_description("Get current timestamp in nanoseconds".to_string()),
        |_args| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64;
            Ok(WasmValue::I64(timestamp))
        },
    ).with_description("Get current timestamp in nanoseconds".to_string());
    
    functions.push(Box::new(timestamp_function));
    
    // 数学函数：加法
    let add_function = SimpleHostFunction::new(
        "add".to_string(),
        WasmFunctionSignature::new(
            "add".to_string(),
            vec![WasmType::F64, WasmType::F64],
            WasmType::F64,
        ).with_description("Add two numbers".to_string()),
        |args| {
            let a = args[0].as_f64()?;
            let b = args[1].as_f64()?;
            Ok(WasmValue::F64(a + b))
        },
    ).with_description("Add two numbers".to_string());
    
    functions.push(Box::new(add_function));
    
    // 字符串长度函数
    let strlen_function = SimpleHostFunction::new(
        "strlen".to_string(),
        WasmFunctionSignature::new(
            "strlen".to_string(),
            vec![WasmType::String],
            WasmType::I32,
        ).with_description("Get string length".to_string()),
        |args| {
            if let WasmValue::String(s) = &args[0] {
                Ok(WasmValue::I32(s.len() as i32))
            } else {
                Err(Error::type_error("Expected string argument"))
            }
        },
    ).with_description("Get string length".to_string());
    
    functions.push(Box::new(strlen_function));
    
    functions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_bridge_creation() {
        let bridge = WasmBridge::new();
        assert_eq!(bridge.function_count(), 0);
        assert!(bridge.list_host_functions().is_empty());
    }

    #[test]
    fn test_host_function_registration() {
        let bridge = WasmBridge::new();
        
        let function = SimpleHostFunction::new(
            "test_function".to_string(),
            WasmFunctionSignature::new(
                "test_function".to_string(),
                vec![WasmType::I32],
                WasmType::I32,
            ),
            |args| {
                let value = args[0].as_i32()?;
                Ok(WasmValue::I32(value * 2))
            },
        );
        
        let result = bridge.register_host_function(Box::new(function));
        assert!(result.is_ok());
        assert_eq!(bridge.function_count(), 1);
        assert!(bridge.has_function("test_function"));
        
        // 测试重复注册
        let duplicate_function = SimpleHostFunction::new(
            "test_function".to_string(),
            WasmFunctionSignature::new(
                "test_function".to_string(),
                vec![WasmType::I32],
                WasmType::I32,
            ),
            |args| Ok(WasmValue::I32(0)),
        );
        
        let result = bridge.register_host_function(Box::new(duplicate_function));
        assert!(result.is_err());
    }

    #[test]
    fn test_host_function_call() {
        let bridge = WasmBridge::new();
        
        let function = SimpleHostFunction::new(
            "double".to_string(),
            WasmFunctionSignature::new(
                "double".to_string(),
                vec![WasmType::I32],
                WasmType::I32,
            ),
            |args| {
                let value = args[0].as_i32()?;
                Ok(WasmValue::I32(value * 2))
            },
        );
        
        bridge.register_host_function(Box::new(function)).unwrap();
        
        let args = vec![WasmValue::I32(21)];
        let result = bridge.call_host_function("double", &args).unwrap();
        assert_eq!(result, WasmValue::I32(42));
        
        // 检查统计
        let stats = bridge.get_function_stats("double").unwrap();
        assert_eq!(stats.call_count, 1);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.error_count, 0);
    }

    #[test]
    fn test_standard_host_functions() {
        let functions = create_standard_host_functions();
        assert!(!functions.is_empty());
        
        let bridge = WasmBridge::new();
        for function in functions {
            bridge.register_host_function(function).unwrap();
        }
        
        assert!(bridge.has_function("log"));
        assert!(bridge.has_function("timestamp"));
        assert!(bridge.has_function("add"));
        assert!(bridge.has_function("strlen"));
        
        // 测试调用
        let args = vec![WasmValue::F64(1.5), WasmValue::F64(2.5)];
        let result = bridge.call_host_function("add", &args).unwrap();
        assert_eq!(result, WasmValue::F64(4.0));
    }

    #[test]
    fn test_function_stats() {
        let mut stats = FunctionCallStats::default();
        
        stats.record_call(100, true, None);
        stats.record_call(200, false, Some("test error".to_string()));
        
        assert_eq!(stats.call_count, 2);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.success_rate(), 0.5);
        assert_eq!(stats.average_execution_time_ms, 150.0);
        assert_eq!(stats.last_error, Some("test error".to_string()));
    }
}
