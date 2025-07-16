//! WASM metrics collection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// WASM指标收集器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WasmMetrics {
    /// 模块加载次数
    pub modules_loaded: u64,
    /// 模块卸载次数
    pub modules_unloaded: u64,
    /// 当前加载的模块数
    pub current_modules: u64,
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
    /// 最小执行时间（毫秒）
    pub min_execution_time_ms: u64,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 内存使用峰值
    pub peak_memory_usage: usize,
    /// 当前内存使用
    pub current_memory_usage: usize,
    /// 超时次数
    pub timeout_count: u64,
    /// 安全违规次数
    pub security_violations: u64,
    /// 热加载次数
    pub hot_reload_count: u64,
    /// 按模块统计
    pub module_stats: HashMap<String, ModuleMetrics>,
    /// 按函数统计
    pub function_stats: HashMap<String, FunctionMetrics>,
    /// 启动时间
    pub start_time: Option<SystemTime>,
    /// 最后更新时间
    pub last_updated: Option<SystemTime>,
}

/// 模块指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleMetrics {
    /// 模块名称
    pub name: String,
    /// 加载次数
    pub load_count: u64,
    /// 卸载次数
    pub unload_count: u64,
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
    /// 当前内存使用
    pub current_memory_usage: usize,
    /// 最后调用时间
    pub last_call_time: Option<SystemTime>,
    /// 错误次数
    pub error_count: u64,
    /// 最后错误时间
    pub last_error_time: Option<SystemTime>,
}

/// 函数指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionMetrics {
    /// 函数名称
    pub name: String,
    /// 模块名称
    pub module_name: String,
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
    /// 最小执行时间（毫秒）
    pub min_execution_time_ms: u64,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 最后调用时间
    pub last_call_time: Option<SystemTime>,
    /// 最后错误时间
    pub last_error_time: Option<SystemTime>,
}

impl WasmMetrics {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            start_time: Some(SystemTime::now()),
            ..Default::default()
        }
    }
    
    /// 记录模块加载
    pub fn record_module_loaded(&mut self) {
        self.modules_loaded += 1;
        self.current_modules += 1;
        self.last_updated = Some(SystemTime::now());
    }
    
    /// 记录模块卸载
    pub fn record_module_unloaded(&mut self) {
        self.modules_unloaded += 1;
        if self.current_modules > 0 {
            self.current_modules -= 1;
        }
        self.last_updated = Some(SystemTime::now());
    }
    
    /// 记录函数调用
    pub fn record_function_call(&mut self, execution_time_ms: u64, success: bool, memory_usage: usize) {
        self.function_calls += 1;
        self.total_execution_time_ms += execution_time_ms;
        
        if success {
            self.successful_calls += 1;
        } else {
            self.failed_calls += 1;
        }
        
        // 更新执行时间统计
        if self.function_calls == 1 {
            self.min_execution_time_ms = execution_time_ms;
            self.max_execution_time_ms = execution_time_ms;
        } else {
            self.min_execution_time_ms = self.min_execution_time_ms.min(execution_time_ms);
            self.max_execution_time_ms = self.max_execution_time_ms.max(execution_time_ms);
        }
        
        self.average_execution_time_ms = self.total_execution_time_ms as f64 / self.function_calls as f64;
        
        // 更新内存使用
        self.current_memory_usage = memory_usage;
        if memory_usage > self.peak_memory_usage {
            self.peak_memory_usage = memory_usage;
        }
        
        self.last_updated = Some(SystemTime::now());
    }
    
    /// 记录模块函数调用
    pub fn record_module_function_call(
        &mut self,
        module_name: &str,
        function_name: &str,
        execution_time_ms: u64,
        success: bool,
        memory_usage: usize,
    ) {
        // 更新全局统计
        self.record_function_call(execution_time_ms, success, memory_usage);
        
        // 更新模块统计
        let module_stats = self.module_stats.entry(module_name.to_string()).or_insert_with(|| {
            ModuleMetrics {
                name: module_name.to_string(),
                ..Default::default()
            }
        });
        
        module_stats.function_calls += 1;
        module_stats.total_execution_time_ms += execution_time_ms;
        module_stats.current_memory_usage = memory_usage;
        module_stats.last_call_time = Some(SystemTime::now());
        
        if success {
            module_stats.successful_calls += 1;
        } else {
            module_stats.failed_calls += 1;
            module_stats.error_count += 1;
            module_stats.last_error_time = Some(SystemTime::now());
        }
        
        if memory_usage > module_stats.peak_memory_usage {
            module_stats.peak_memory_usage = memory_usage;
        }
        
        module_stats.average_execution_time_ms = 
            module_stats.total_execution_time_ms as f64 / module_stats.function_calls as f64;
        
        // 更新函数统计
        let function_key = format!("{}::{}", module_name, function_name);
        let function_stats = self.function_stats.entry(function_key).or_insert_with(|| {
            FunctionMetrics {
                name: function_name.to_string(),
                module_name: module_name.to_string(),
                min_execution_time_ms: execution_time_ms,
                max_execution_time_ms: execution_time_ms,
                ..Default::default()
            }
        });
        
        function_stats.call_count += 1;
        function_stats.total_execution_time_ms += execution_time_ms;
        function_stats.last_call_time = Some(SystemTime::now());
        
        if success {
            function_stats.success_count += 1;
        } else {
            function_stats.error_count += 1;
            function_stats.last_error_time = Some(SystemTime::now());
        }
        
        function_stats.min_execution_time_ms = function_stats.min_execution_time_ms.min(execution_time_ms);
        function_stats.max_execution_time_ms = function_stats.max_execution_time_ms.max(execution_time_ms);
        function_stats.average_execution_time_ms = 
            function_stats.total_execution_time_ms as f64 / function_stats.call_count as f64;
    }
    
    /// 记录超时
    pub fn record_timeout(&mut self) {
        self.timeout_count += 1;
        self.failed_calls += 1;
        self.last_updated = Some(SystemTime::now());
    }
    
    /// 记录安全违规
    pub fn record_security_violation(&mut self) {
        self.security_violations += 1;
        self.last_updated = Some(SystemTime::now());
    }
    
    /// 记录热加载
    pub fn record_hot_reload(&mut self) {
        self.hot_reload_count += 1;
        self.last_updated = Some(SystemTime::now());
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.function_calls == 0 {
            0.0
        } else {
            self.successful_calls as f64 / self.function_calls as f64
        }
    }
    
    /// 获取运行时间
    pub fn uptime(&self) -> Option<Duration> {
        self.start_time.and_then(|start| SystemTime::now().duration_since(start).ok())
    }
    
    /// 获取每秒调用次数
    pub fn calls_per_second(&self) -> f64 {
        if let Some(uptime) = self.uptime() {
            let seconds = uptime.as_secs_f64();
            if seconds > 0.0 {
                return self.function_calls as f64 / seconds;
            }
        }
        0.0
    }
    
    /// 获取模块统计
    pub fn get_module_stats(&self, module_name: &str) -> Option<&ModuleMetrics> {
        self.module_stats.get(module_name)
    }
    
    /// 获取函数统计
    pub fn get_function_stats(&self, module_name: &str, function_name: &str) -> Option<&FunctionMetrics> {
        let function_key = format!("{}::{}", module_name, function_name);
        self.function_stats.get(&function_key)
    }
    
    /// 获取热门模块（按调用次数排序）
    pub fn get_top_modules(&self, limit: usize) -> Vec<&ModuleMetrics> {
        let mut modules: Vec<&ModuleMetrics> = self.module_stats.values().collect();
        modules.sort_by(|a, b| b.function_calls.cmp(&a.function_calls));
        modules.into_iter().take(limit).collect()
    }
    
    /// 获取热门函数（按调用次数排序）
    pub fn get_top_functions(&self, limit: usize) -> Vec<&FunctionMetrics> {
        let mut functions: Vec<&FunctionMetrics> = self.function_stats.values().collect();
        functions.sort_by(|a, b| b.call_count.cmp(&a.call_count));
        functions.into_iter().take(limit).collect()
    }
    
    /// 获取慢函数（按平均执行时间排序）
    pub fn get_slowest_functions(&self, limit: usize) -> Vec<&FunctionMetrics> {
        let mut functions: Vec<&FunctionMetrics> = self.function_stats.values().collect();
        functions.sort_by(|a, b| b.average_execution_time_ms.partial_cmp(&a.average_execution_time_ms).unwrap_or(std::cmp::Ordering::Equal));
        functions.into_iter().take(limit).collect()
    }
    
    /// 重置所有统计
    pub fn reset(&mut self) {
        *self = WasmMetrics::new();
    }
    
    /// 重置模块统计
    pub fn reset_module_stats(&mut self, module_name: &str) {
        self.module_stats.remove(module_name);
    }
    
    /// 生成报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== WASM Runtime Metrics Report ===\n\n");
        
        // 基本统计
        report.push_str(&format!("Modules loaded: {}\n", self.modules_loaded));
        report.push_str(&format!("Current modules: {}\n", self.current_modules));
        report.push_str(&format!("Function calls: {}\n", self.function_calls));
        report.push_str(&format!("Success rate: {:.2}%\n", self.success_rate() * 100.0));
        report.push_str(&format!("Average execution time: {:.2}ms\n", self.average_execution_time_ms));
        report.push_str(&format!("Calls per second: {:.2}\n", self.calls_per_second()));
        report.push_str(&format!("Peak memory usage: {} bytes\n", self.peak_memory_usage));
        report.push_str(&format!("Timeouts: {}\n", self.timeout_count));
        report.push_str(&format!("Security violations: {}\n", self.security_violations));
        
        if let Some(uptime) = self.uptime() {
            report.push_str(&format!("Uptime: {:.2}s\n", uptime.as_secs_f64()));
        }
        
        report.push_str("\n=== Top Modules ===\n");
        for (i, module) in self.get_top_modules(5).iter().enumerate() {
            report.push_str(&format!("{}. {} - {} calls ({:.2}% success)\n", 
                i + 1, module.name, module.function_calls, 
                if module.function_calls > 0 { module.successful_calls as f64 / module.function_calls as f64 * 100.0 } else { 0.0 }
            ));
        }
        
        report.push_str("\n=== Slowest Functions ===\n");
        for (i, function) in self.get_slowest_functions(5).iter().enumerate() {
            report.push_str(&format!("{}. {}::{} - {:.2}ms avg\n", 
                i + 1, function.module_name, function.name, function.average_execution_time_ms
            ));
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_metrics_creation() {
        let metrics = WasmMetrics::new();
        assert_eq!(metrics.modules_loaded, 0);
        assert_eq!(metrics.function_calls, 0);
        assert!(metrics.start_time.is_some());
    }

    #[test]
    fn test_module_operations() {
        let mut metrics = WasmMetrics::new();
        
        metrics.record_module_loaded();
        assert_eq!(metrics.modules_loaded, 1);
        assert_eq!(metrics.current_modules, 1);
        
        metrics.record_module_unloaded();
        assert_eq!(metrics.modules_unloaded, 1);
        assert_eq!(metrics.current_modules, 0);
    }

    #[test]
    fn test_function_call_recording() {
        let mut metrics = WasmMetrics::new();
        
        metrics.record_function_call(100, true, 1024);
        metrics.record_function_call(200, false, 2048);
        
        assert_eq!(metrics.function_calls, 2);
        assert_eq!(metrics.successful_calls, 1);
        assert_eq!(metrics.failed_calls, 1);
        assert_eq!(metrics.success_rate(), 0.5);
        assert_eq!(metrics.average_execution_time_ms, 150.0);
        assert_eq!(metrics.min_execution_time_ms, 100);
        assert_eq!(metrics.max_execution_time_ms, 200);
        assert_eq!(metrics.peak_memory_usage, 2048);
    }

    #[test]
    fn test_module_function_call_recording() {
        let mut metrics = WasmMetrics::new();
        
        metrics.record_module_function_call("test_module", "test_function", 100, true, 1024);
        
        assert_eq!(metrics.function_calls, 1);
        assert!(metrics.module_stats.contains_key("test_module"));
        assert!(metrics.function_stats.contains_key("test_module::test_function"));
        
        let module_stats = metrics.get_module_stats("test_module").unwrap();
        assert_eq!(module_stats.function_calls, 1);
        assert_eq!(module_stats.successful_calls, 1);
        
        let function_stats = metrics.get_function_stats("test_module", "test_function").unwrap();
        assert_eq!(function_stats.call_count, 1);
        assert_eq!(function_stats.success_count, 1);
    }

    #[test]
    fn test_metrics_report() {
        let mut metrics = WasmMetrics::new();
        
        metrics.record_module_loaded();
        metrics.record_module_function_call("test_module", "test_function", 100, true, 1024);
        
        let report = metrics.generate_report();
        assert!(report.contains("WASM Runtime Metrics Report"));
        assert!(report.contains("Modules loaded: 1"));
        assert!(report.contains("Function calls: 1"));
    }
}
