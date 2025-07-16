//! WASM event system

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

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
    /// 插件热加载
    PluginHotReloaded {
        plugin_name: String,
        old_version: String,
        new_version: String,
    },
    /// 插件错误
    PluginError {
        plugin_name: String,
        error_message: String,
    },
}

impl WasmEvent {
    /// 获取事件时间戳
    pub fn timestamp(&self) -> SystemTime {
        SystemTime::now()
    }
    
    /// 获取事件类型名称
    pub fn event_type(&self) -> &'static str {
        match self {
            WasmEvent::ModuleLoaded { .. } => "module_loaded",
            WasmEvent::ModuleUnloaded { .. } => "module_unloaded",
            WasmEvent::FunctionCallStarted { .. } => "function_call_started",
            WasmEvent::FunctionCallCompleted { .. } => "function_call_completed",
            WasmEvent::MemoryLimitExceeded { .. } => "memory_limit_exceeded",
            WasmEvent::ExecutionTimeout { .. } => "execution_timeout",
            WasmEvent::SecurityViolation { .. } => "security_violation",
            WasmEvent::PluginHotReloaded { .. } => "plugin_hot_reloaded",
            WasmEvent::PluginError { .. } => "plugin_error",
        }
    }
    
    /// 获取模块名称
    pub fn module_name(&self) -> Option<&str> {
        match self {
            WasmEvent::ModuleLoaded { module_name, .. } => Some(module_name),
            WasmEvent::ModuleUnloaded { module_name } => Some(module_name),
            WasmEvent::FunctionCallStarted { module_name, .. } => Some(module_name),
            WasmEvent::FunctionCallCompleted { module_name, .. } => Some(module_name),
            WasmEvent::MemoryLimitExceeded { module_name, .. } => Some(module_name),
            WasmEvent::ExecutionTimeout { module_name, .. } => Some(module_name),
            WasmEvent::SecurityViolation { module_name, .. } => Some(module_name),
            WasmEvent::PluginHotReloaded { plugin_name, .. } => Some(plugin_name),
            WasmEvent::PluginError { plugin_name, .. } => Some(plugin_name),
        }
    }
    
    /// 检查是否为错误事件
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            WasmEvent::MemoryLimitExceeded { .. }
                | WasmEvent::ExecutionTimeout { .. }
                | WasmEvent::SecurityViolation { .. }
                | WasmEvent::PluginError { .. }
        )
    }
    
    /// 检查是否为性能相关事件
    pub fn is_performance_related(&self) -> bool {
        matches!(
            self,
            WasmEvent::FunctionCallCompleted { .. }
                | WasmEvent::MemoryLimitExceeded { .. }
                | WasmEvent::ExecutionTimeout { .. }
        )
    }
}

/// WASM事件监听器特征
pub trait WasmEventListener: Send + Sync {
    /// 处理WASM事件
    fn on_event(&self, event: WasmEvent);
    
    /// 获取监听器名称
    fn name(&self) -> &str {
        "unnamed_listener"
    }
    
    /// 检查是否对特定事件类型感兴趣
    fn is_interested_in(&self, event_type: &str) -> bool {
        // 默认对所有事件感兴趣
        true
    }
}

/// 控制台事件监听器
pub struct ConsoleEventListener {
    name: String,
    verbose: bool,
}

impl ConsoleEventListener {
    pub fn new(name: String, verbose: bool) -> Self {
        Self { name, verbose }
    }
}

impl WasmEventListener for ConsoleEventListener {
    fn on_event(&self, event: WasmEvent) {
        let timestamp = event.timestamp();
        let event_type = event.event_type();
        
        if self.verbose || event.is_error() {
            match event {
                WasmEvent::ModuleLoaded { module_name, module_size } => {
                    println!("[{}] Module loaded: {} ({} bytes)", 
                        format_timestamp(timestamp), module_name, module_size);
                }
                WasmEvent::ModuleUnloaded { module_name } => {
                    println!("[{}] Module unloaded: {}", 
                        format_timestamp(timestamp), module_name);
                }
                WasmEvent::FunctionCallCompleted { module_name, function_name, execution_time_ms, success } => {
                    if self.verbose {
                        println!("[{}] Function call completed: {}::{} ({}ms, success: {})", 
                            format_timestamp(timestamp), module_name, function_name, execution_time_ms, success);
                    }
                }
                WasmEvent::MemoryLimitExceeded { module_name, current_usage, limit } => {
                    eprintln!("[{}] ERROR: Memory limit exceeded in {}: {} > {}", 
                        format_timestamp(timestamp), module_name, current_usage, limit);
                }
                WasmEvent::ExecutionTimeout { module_name, function_name, timeout_ms } => {
                    eprintln!("[{}] ERROR: Execution timeout in {}::{} ({}ms)", 
                        format_timestamp(timestamp), module_name, function_name, timeout_ms);
                }
                WasmEvent::SecurityViolation { module_name, violation_type, details } => {
                    eprintln!("[{}] SECURITY: {} in {}: {}", 
                        format_timestamp(timestamp), violation_type, module_name, details);
                }
                WasmEvent::PluginHotReloaded { plugin_name, old_version, new_version } => {
                    println!("[{}] Plugin hot reloaded: {} ({} -> {})", 
                        format_timestamp(timestamp), plugin_name, old_version, new_version);
                }
                WasmEvent::PluginError { plugin_name, error_message } => {
                    eprintln!("[{}] Plugin error in {}: {}", 
                        format_timestamp(timestamp), plugin_name, error_message);
                }
                _ => {
                    if self.verbose {
                        println!("[{}] Event: {}", format_timestamp(timestamp), event_type);
                    }
                }
            }
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 格式化时间戳
fn format_timestamp(timestamp: SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    
    match timestamp.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let nanos = duration.subsec_nanos();
            format!("{}.{:09}", secs, nanos)
        }
        Err(_) => "invalid_timestamp".to_string(),
    }
}

/// 文件事件监听器
pub struct FileEventListener {
    name: String,
    log_file: std::path::PathBuf,
}

impl FileEventListener {
    pub fn new(name: String, log_file: std::path::PathBuf) -> Self {
        Self { name, log_file }
    }
}

impl WasmEventListener for FileEventListener {
    fn on_event(&self, event: WasmEvent) {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let timestamp = event.timestamp();
        let event_json = serde_json::to_string(&event).unwrap_or_else(|_| "invalid_event".to_string());
        let log_line = format!("[{}] {}\n", format_timestamp(timestamp), event_json);
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 指标事件监听器
pub struct MetricsEventListener {
    name: String,
    metrics: std::sync::Arc<parking_lot::RwLock<crate::metrics::WasmMetrics>>,
}

impl MetricsEventListener {
    pub fn new(name: String, metrics: std::sync::Arc<parking_lot::RwLock<crate::metrics::WasmMetrics>>) -> Self {
        Self { name, metrics }
    }
}

impl WasmEventListener for MetricsEventListener {
    fn on_event(&self, event: WasmEvent) {
        let mut metrics = self.metrics.write();
        
        match event {
            WasmEvent::ModuleLoaded { .. } => {
                metrics.record_module_loaded();
            }
            WasmEvent::ModuleUnloaded { .. } => {
                metrics.record_module_unloaded();
            }
            WasmEvent::FunctionCallCompleted { execution_time_ms, success, .. } => {
                metrics.record_function_call(execution_time_ms, success, 0);
            }
            WasmEvent::ExecutionTimeout { .. } => {
                metrics.record_timeout();
            }
            WasmEvent::SecurityViolation { .. } => {
                metrics.record_security_violation();
            }
            _ => {}
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn is_interested_in(&self, event_type: &str) -> bool {
        matches!(
            event_type,
            "module_loaded" | "module_unloaded" | "function_call_completed" 
            | "execution_timeout" | "security_violation"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_event_properties() {
        let event = WasmEvent::ModuleLoaded {
            module_name: "test_module".to_string(),
            module_size: 1024,
        };
        
        assert_eq!(event.event_type(), "module_loaded");
        assert_eq!(event.module_name(), Some("test_module"));
        assert!(!event.is_error());
        assert!(!event.is_performance_related());
    }

    #[test]
    fn test_error_events() {
        let event = WasmEvent::MemoryLimitExceeded {
            module_name: "test_module".to_string(),
            current_usage: 1024,
            limit: 512,
        };
        
        assert!(event.is_error());
        assert!(event.is_performance_related());
    }

    #[test]
    fn test_console_event_listener() {
        let listener = ConsoleEventListener::new("test_listener".to_string(), true);
        assert_eq!(listener.name(), "test_listener");
        assert!(listener.is_interested_in("any_event"));
        
        let event = WasmEvent::ModuleLoaded {
            module_name: "test_module".to_string(),
            module_size: 1024,
        };
        
        // 这个测试只是确保不会panic
        listener.on_event(event);
    }

    #[test]
    fn test_event_serialization() {
        let event = WasmEvent::FunctionCallCompleted {
            module_name: "test_module".to_string(),
            function_name: "test_function".to_string(),
            execution_time_ms: 100,
            success: true,
        };
        
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: WasmEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.event_type(), deserialized.event_type());
        assert_eq!(event.module_name(), deserialized.module_name());
    }
}
