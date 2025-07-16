//! WASM runtime implementation

use crate::{
    security::SecurityPolicy,
    types::WasmValue,
    events::{WasmEvent, WasmEventListener},
    metrics::WasmMetrics,
};
use fdc_core::{
    error::{Error, Result},
};
use wasmtime::{Engine, Store, Module, Instance, Linker};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// WASM运行时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmRuntimeConfig {
    /// 内存限制（字节）
    pub memory_limit: usize,
    /// 执行超时（毫秒）
    pub execution_timeout_ms: u64,
    /// 最大插件数量
    pub max_plugins: usize,
    /// 是否启用WASI
    pub enable_wasi: bool,
    /// 是否启用多线程
    pub enable_threads: bool,
    /// 是否启用SIMD
    pub enable_simd: bool,
    /// 安全策略
    pub security_policy: SecurityPolicy,
}

impl Default for WasmRuntimeConfig {
    fn default() -> Self {
        Self {
            memory_limit: crate::DEFAULT_MEMORY_LIMIT,
            execution_timeout_ms: crate::DEFAULT_EXECUTION_TIMEOUT_MS,
            max_plugins: crate::DEFAULT_MAX_PLUGINS,
            enable_wasi: true,
            enable_threads: false,
            enable_simd: true,
            security_policy: SecurityPolicy::default(),
        }
    }
}

/// WASM运行时状态
#[derive(Debug)]
struct WasmState {
    security_policy: SecurityPolicy,
    start_time: Instant,
    memory_usage: usize,
}

/// WASM运行时实现
pub struct WasmRuntime {
    engine: Engine,
    config: WasmRuntimeConfig,
    modules: Arc<RwLock<HashMap<String, Module>>>,
    instances: Arc<RwLock<HashMap<String, Instance>>>,
    linker: Linker<WasmState>,
    event_listeners: Arc<RwLock<Vec<Box<dyn WasmEventListener>>>>,
    metrics: Arc<RwLock<WasmMetrics>>,
}

impl WasmRuntime {
    /// 创建新的WASM运行时
    pub fn new(config: WasmRuntimeConfig) -> Result<Self> {
        // 配置Wasmtime引擎
        let mut engine_config = wasmtime::Config::new();
        engine_config.wasm_simd(config.enable_simd);
        engine_config.wasm_threads(config.enable_threads);
        engine_config.consume_fuel(true);
        
        let engine = Engine::new(&engine_config)
            .map_err(|e| Error::wasm(format!("Failed to create WASM engine: {}", e)))?;
        
        // 创建链接器
        let mut linker = Linker::new(&engine);
        
        // TODO: 添加WASI支持（暂时跳过以简化实现）
        // if config.enable_wasi {
        //     wasmtime_wasi::add_to_linker(&mut linker, |state: &mut WasmState| state)
        //         .map_err(|e| Error::wasm(format!("Failed to add WASI to linker: {}", e)))?;
        // }
        
        Ok(Self {
            engine,
            config,
            modules: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            linker,
            event_listeners: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(WasmMetrics::default())),
        })
    }
    
    /// 加载WASM模块
    pub fn load_module(&self, name: &str, wasm_bytes: &[u8]) -> Result<()> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| Error::wasm(format!("Failed to compile WASM module {}: {}", name, e)))?;
        
        self.modules.write().insert(name.to_string(), module);
        
        // 触发事件
        self.emit_event(WasmEvent::ModuleLoaded {
            module_name: name.to_string(),
            module_size: wasm_bytes.len(),
        });
        
        // 更新指标
        self.metrics.write().record_module_loaded();
        
        Ok(())
    }
    
    /// 卸载WASM模块
    pub fn unload_module(&self, name: &str) -> Result<()> {
        let mut modules = self.modules.write();
        let mut instances = self.instances.write();
        
        if modules.remove(name).is_none() {
            return Err(Error::not_found(format!("Module: {}", name)));
        }
        
        instances.remove(name);
        
        // 触发事件
        self.emit_event(WasmEvent::ModuleUnloaded {
            module_name: name.to_string(),
        });
        
        // 更新指标
        self.metrics.write().record_module_unloaded();
        
        Ok(())
    }
    
    /// 实例化模块
    pub fn instantiate_module(&self, name: &str) -> Result<()> {
        let modules = self.modules.read();
        let module = modules.get(name)
            .ok_or_else(|| Error::not_found(format!("Module: {}", name)))?;
        
        // 创建WASM状态
        let state = WasmState {
            security_policy: self.config.security_policy.clone(),
            start_time: Instant::now(),
            memory_usage: 0,
        };
        
        let mut store = Store::new(&self.engine, state);
        
        // 设置燃料限制（用于执行超时）
        store.set_fuel(1_000_000)
            .map_err(|e| Error::wasm(format!("Failed to set fuel: {}", e)))?;
        
        // 实例化模块
        let instance = self.linker.instantiate(&mut store, module)
            .map_err(|e| Error::wasm(format!("Failed to instantiate module {}: {}", name, e)))?;
        
        self.instances.write().insert(name.to_string(), instance);
        
        Ok(())
    }
    
    /// 调用WASM函数
    pub fn call_function(
        &self,
        module_name: &str,
        function_name: &str,
        args: &[WasmValue],
    ) -> Result<WasmValue> {
        let start_time = Instant::now();
        
        // 触发函数调用开始事件
        self.emit_event(WasmEvent::FunctionCallStarted {
            module_name: module_name.to_string(),
            function_name: function_name.to_string(),
            args_count: args.len(),
        });
        
        // 检查模块是否存在
        let instances = self.instances.read();
        let instance = instances.get(module_name)
            .ok_or_else(|| Error::not_found(format!("Module instance: {}", module_name)))?;
        
        // 这里需要实际的函数调用实现
        // 由于wasmtime API的复杂性，这里提供一个简化的实现框架
        let result = self.execute_function_call(instance, function_name, args);
        
        let execution_time = start_time.elapsed();
        let success = result.is_ok();
        
        // 触发函数调用完成事件
        self.emit_event(WasmEvent::FunctionCallCompleted {
            module_name: module_name.to_string(),
            function_name: function_name.to_string(),
            execution_time_ms: execution_time.as_millis() as u64,
            success,
        });
        
        // 更新指标
        self.metrics.write().record_function_call(
            execution_time.as_millis() as u64,
            success,
            0, // 内存使用量需要从store中获取
        );
        
        result
    }
    
    /// 执行函数调用（简化实现）
    fn execute_function_call(
        &self,
        _instance: &Instance,
        _function_name: &str,
        _args: &[WasmValue],
    ) -> Result<WasmValue> {
        // TODO: 实现实际的函数调用逻辑
        // 这需要处理类型转换、参数传递、结果获取等
        Ok(WasmValue::Null)
    }
    
    /// 添加事件监听器
    pub fn add_event_listener(&self, listener: Box<dyn WasmEventListener>) {
        self.event_listeners.write().push(listener);
    }
    
    /// 触发事件
    fn emit_event(&self, event: WasmEvent) {
        let listeners = self.event_listeners.read();
        for listener in listeners.iter() {
            listener.on_event(event.clone());
        }
    }
    
    /// 获取运行时配置
    pub fn config(&self) -> &WasmRuntimeConfig {
        &self.config
    }
    
    /// 获取指标
    pub fn metrics(&self) -> WasmMetrics {
        self.metrics.read().clone()
    }
    
    /// 列出所有模块
    pub fn list_modules(&self) -> Vec<String> {
        self.modules.read().keys().cloned().collect()
    }
    
    /// 检查模块是否存在
    pub fn has_module(&self, name: &str) -> bool {
        self.modules.read().contains_key(name)
    }
    
    /// 获取模块数量
    pub fn module_count(&self) -> usize {
        self.modules.read().len()
    }
    
    /// 清理所有模块
    pub fn clear_modules(&self) -> Result<()> {
        let module_names: Vec<String> = self.list_modules();
        for name in module_names {
            self.unload_module(&name)?;
        }
        Ok(())
    }
}

impl Drop for WasmRuntime {
    fn drop(&mut self) {
        // 清理资源
        if let Err(e) = self.clear_modules() {
            tracing::error!("Failed to clear modules during runtime drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let config = WasmRuntimeConfig::default();
        let runtime = WasmRuntime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_runtime_config() {
        let config = WasmRuntimeConfig::default();
        assert_eq!(config.memory_limit, crate::DEFAULT_MEMORY_LIMIT);
        assert_eq!(config.execution_timeout_ms, crate::DEFAULT_EXECUTION_TIMEOUT_MS);
        assert_eq!(config.max_plugins, crate::DEFAULT_MAX_PLUGINS);
        assert!(config.enable_wasi);
        assert!(config.enable_simd);
        assert!(!config.enable_threads);
    }

    #[test]
    fn test_module_management() {
        let config = WasmRuntimeConfig::default();
        let runtime = WasmRuntime::new(config).unwrap();
        
        assert_eq!(runtime.module_count(), 0);
        assert!(!runtime.has_module("test"));
        assert!(runtime.list_modules().is_empty());
    }
}
