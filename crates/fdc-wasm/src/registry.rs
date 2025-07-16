//! WASM plugin registry

use crate::{
    plugin::{WasmPlugin, PluginInfo, PluginStatus, PluginType},
    runtime::{WasmRuntime, WasmRuntimeConfig},
    security::SecurityPolicy,
    events::{WasmEvent, WasmEventListener},
    metrics::WasmMetrics,
};
use fdc_core::error::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

/// 插件注册表
pub struct PluginRegistry {
    /// 已注册的插件
    plugins: Arc<RwLock<HashMap<Uuid, WasmPlugin>>>,
    /// 插件名称到ID的映射
    name_to_id: Arc<RwLock<HashMap<String, Uuid>>>,
    /// WASM运行时
    runtime: Arc<WasmRuntime>,
    /// 事件监听器
    event_listeners: Arc<RwLock<Vec<Box<dyn WasmEventListener>>>>,
    /// 指标收集器
    metrics: Arc<RwLock<WasmMetrics>>,
    /// 最大插件数量
    max_plugins: usize,
}

impl PluginRegistry {
    /// 创建新的插件注册表
    pub fn new(runtime_config: WasmRuntimeConfig) -> Result<Self> {
        let max_plugins = runtime_config.max_plugins;
        let runtime = Arc::new(WasmRuntime::new(runtime_config)?);
        
        Ok(Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            runtime,
            event_listeners: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(WasmMetrics::new())),
            max_plugins,
        })
    }
    
    /// 注册插件
    pub fn register_plugin(&self, mut plugin: WasmPlugin) -> Result<Uuid> {
        let plugins = self.plugins.read();
        if plugins.len() >= self.max_plugins {
            return Err(Error::resource_exhausted("Maximum number of plugins reached"));
        }
        drop(plugins);
        
        // 验证插件
        plugin.validate()?;
        
        let plugin_id = plugin.info().id;
        let plugin_name = plugin.info().name.clone();
        
        // 检查名称是否已存在
        {
            let name_to_id = self.name_to_id.read();
            if name_to_id.contains_key(&plugin_name) {
                return Err(Error::already_exists(format!("Plugin name: {}", plugin_name)));
            }
        }
        
        // 加载WASM模块到运行时
        self.runtime.load_module(&plugin_name, plugin.wasm_bytes())?;
        
        // 实例化模块
        self.runtime.instantiate_module(&plugin_name)?;
        
        // 更新插件状态
        plugin.info_mut().set_status(PluginStatus::Loaded);
        
        // 注册插件
        {
            let mut plugins = self.plugins.write();
            let mut name_to_id = self.name_to_id.write();
            
            plugins.insert(plugin_id, plugin);
            name_to_id.insert(plugin_name.clone(), plugin_id);
        }
        
        // 触发事件
        self.emit_event(WasmEvent::ModuleLoaded {
            module_name: plugin_name,
            module_size: self.get_plugin(plugin_id).unwrap().wasm_bytes().len(),
        });
        
        // 更新指标
        self.metrics.write().record_module_loaded();
        
        Ok(plugin_id)
    }
    
    /// 卸载插件
    pub fn unregister_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let plugin_name = {
            let plugins = self.plugins.read();
            let plugin = plugins.get(&plugin_id)
                .ok_or_else(|| Error::not_found(format!("Plugin ID: {}", plugin_id)))?;
            plugin.info().name.clone()
        };
        
        // 从运行时卸载模块
        self.runtime.unload_module(&plugin_name)?;
        
        // 从注册表移除
        {
            let mut plugins = self.plugins.write();
            let mut name_to_id = self.name_to_id.write();
            
            plugins.remove(&plugin_id);
            name_to_id.remove(&plugin_name);
        }
        
        // 触发事件
        self.emit_event(WasmEvent::ModuleUnloaded {
            module_name: plugin_name,
        });
        
        // 更新指标
        self.metrics.write().record_module_unloaded();
        
        Ok(())
    }
    
    /// 根据ID获取插件
    pub fn get_plugin(&self, plugin_id: Uuid) -> Option<WasmPlugin> {
        self.plugins.read().get(&plugin_id).cloned()
    }
    
    /// 根据名称获取插件
    pub fn get_plugin_by_name(&self, name: &str) -> Option<WasmPlugin> {
        let name_to_id = self.name_to_id.read();
        let plugin_id = name_to_id.get(name)?;
        self.get_plugin(*plugin_id)
    }
    
    /// 获取插件信息
    pub fn get_plugin_info(&self, plugin_id: Uuid) -> Option<PluginInfo> {
        self.plugins.read().get(&plugin_id).map(|p| p.info().clone())
    }
    
    /// 列出所有插件
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.read()
            .values()
            .map(|p| p.info().clone())
            .collect()
    }
    
    /// 按类型列出插件
    pub fn list_plugins_by_type(&self, plugin_type: PluginType) -> Vec<PluginInfo> {
        self.plugins.read()
            .values()
            .filter(|p| p.info().plugin_type == plugin_type)
            .map(|p| p.info().clone())
            .collect()
    }
    
    /// 按状态列出插件
    pub fn list_plugins_by_status(&self, status: PluginStatus) -> Vec<PluginInfo> {
        self.plugins.read()
            .values()
            .filter(|p| p.info().status == status)
            .map(|p| p.info().clone())
            .collect()
    }
    
    /// 调用插件函数
    pub fn call_plugin_function(
        &self,
        plugin_id: Uuid,
        function_name: &str,
        args: &[crate::types::WasmValue],
    ) -> Result<crate::types::WasmValue> {
        let mut plugin = {
            let plugins = self.plugins.read();
            plugins.get(&plugin_id)
                .ok_or_else(|| Error::not_found(format!("Plugin ID: {}", plugin_id)))?
                .clone()
        };
        
        let result = plugin.call_function(&self.runtime, function_name, args);
        
        // 更新插件状态
        {
            let mut plugins = self.plugins.write();
            if let Some(stored_plugin) = plugins.get_mut(&plugin_id) {
                *stored_plugin = plugin;
            }
        }
        
        result
    }
    
    /// 启用插件
    pub fn enable_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let mut plugins = self.plugins.write();
        let plugin = plugins.get_mut(&plugin_id)
            .ok_or_else(|| Error::not_found(format!("Plugin ID: {}", plugin_id)))?;
        
        if plugin.info().status == PluginStatus::Paused {
            plugin.info_mut().set_status(PluginStatus::Loaded);
            Ok(())
        } else {
            Err(Error::invalid_argument("Plugin is not in paused state"))
        }
    }
    
    /// 禁用插件
    pub fn disable_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let mut plugins = self.plugins.write();
        let plugin = plugins.get_mut(&plugin_id)
            .ok_or_else(|| Error::not_found(format!("Plugin ID: {}", plugin_id)))?;
        
        if plugin.info().can_run() {
            plugin.info_mut().set_status(PluginStatus::Paused);
            Ok(())
        } else {
            Err(Error::invalid_argument("Plugin cannot be disabled in current state"))
        }
    }
    
    /// 重新加载插件
    pub fn reload_plugin(&self, plugin_id: Uuid, new_wasm_bytes: Vec<u8>) -> Result<()> {
        let (plugin_name, old_version) = {
            let plugins = self.plugins.read();
            let plugin = plugins.get(&plugin_id)
                .ok_or_else(|| Error::not_found(format!("Plugin ID: {}", plugin_id)))?;
            (plugin.info().name.clone(), plugin.info().version.clone())
        };
        
        // 卸载旧模块
        self.runtime.unload_module(&plugin_name)?;
        
        // 加载新模块
        self.runtime.load_module(&plugin_name, &new_wasm_bytes)?;
        self.runtime.instantiate_module(&plugin_name)?;
        
        // 更新插件
        {
            let mut plugins = self.plugins.write();
            if let Some(plugin) = plugins.get_mut(&plugin_id) {
                // 这里需要更新插件的WASM字节码
                // 由于WasmPlugin结构的限制，这里简化处理
                plugin.info_mut().set_status(PluginStatus::Loaded);
                plugin.info_mut().version = format!("{}.reloaded", old_version);
            }
        }
        
        // 触发热加载事件
        self.emit_event(WasmEvent::PluginHotReloaded {
            plugin_name: plugin_name.clone(),
            old_version: old_version.clone(),
            new_version: format!("{}.reloaded", old_version),
        });
        
        // 更新指标
        self.metrics.write().record_hot_reload();
        
        Ok(())
    }
    
    /// 获取插件统计信息
    pub fn get_plugin_stats(&self, plugin_id: Uuid) -> Option<crate::plugin::PluginStats> {
        self.plugins.read()
            .get(&plugin_id)
            .map(|p| p.stats().clone())
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
    
    /// 获取运行时
    pub fn runtime(&self) -> &Arc<WasmRuntime> {
        &self.runtime
    }
    
    /// 获取指标
    pub fn metrics(&self) -> WasmMetrics {
        self.metrics.read().clone()
    }
    
    /// 获取插件数量
    pub fn plugin_count(&self) -> usize {
        self.plugins.read().len()
    }
    
    /// 检查是否达到最大插件数量
    pub fn is_at_capacity(&self) -> bool {
        self.plugin_count() >= self.max_plugins
    }
    
    /// 清理所有插件
    pub fn clear_all_plugins(&self) -> Result<()> {
        let plugin_ids: Vec<Uuid> = self.plugins.read().keys().cloned().collect();
        for plugin_id in plugin_ids {
            self.unregister_plugin(plugin_id)?;
        }
        Ok(())
    }
    
    /// 验证插件依赖
    pub fn validate_dependencies(&self, plugin_info: &PluginInfo) -> Result<()> {
        for dependency in &plugin_info.dependencies {
            if self.get_plugin_by_name(dependency).is_none() {
                return Err(Error::validation(format!(
                    "Missing dependency: {} for plugin {}",
                    dependency, plugin_info.name
                )));
            }
        }
        Ok(())
    }
    
    /// 获取插件依赖图
    pub fn get_dependency_graph(&self) -> HashMap<String, Vec<String>> {
        let plugins = self.plugins.read();
        let mut graph = HashMap::new();
        
        for plugin in plugins.values() {
            graph.insert(
                plugin.info().name.clone(),
                plugin.info().dependencies.clone(),
            );
        }
        
        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::WasmRuntimeConfig;
    use std::path::PathBuf;

    fn create_test_plugin() -> WasmPlugin {
        let info = PluginInfo::new(
            "test_plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::DataTransform,
            PathBuf::from("/test/plugin.wasm"),
        );
        
        let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d]; // 简单的WASM魔数
        let security_policy = SecurityPolicy::default();
        
        WasmPlugin::new(info, wasm_bytes, security_policy)
    }

    #[test]
    fn test_registry_creation() {
        let config = WasmRuntimeConfig::default();
        let registry = PluginRegistry::new(config);
        assert!(registry.is_ok());
    }

    #[test]
    fn test_plugin_count() {
        let config = WasmRuntimeConfig::default();
        let registry = PluginRegistry::new(config).unwrap();
        
        assert_eq!(registry.plugin_count(), 0);
        assert!(!registry.is_at_capacity());
    }

    #[test]
    fn test_plugin_listing() {
        let config = WasmRuntimeConfig::default();
        let registry = PluginRegistry::new(config).unwrap();
        
        let plugins = registry.list_plugins();
        assert!(plugins.is_empty());
        
        let data_transform_plugins = registry.list_plugins_by_type(PluginType::DataTransform);
        assert!(data_transform_plugins.is_empty());
        
        let loaded_plugins = registry.list_plugins_by_status(PluginStatus::Loaded);
        assert!(loaded_plugins.is_empty());
    }

    #[test]
    fn test_dependency_validation() {
        let config = WasmRuntimeConfig::default();
        let registry = PluginRegistry::new(config).unwrap();
        
        let mut plugin_info = PluginInfo::new(
            "test_plugin".to_string(),
            "1.0.0".to_string(),
            PluginType::DataTransform,
            PathBuf::from("/test/plugin.wasm"),
        );
        
        plugin_info.add_dependency("nonexistent_plugin".to_string());
        
        let result = registry.validate_dependencies(&plugin_info);
        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_graph() {
        let config = WasmRuntimeConfig::default();
        let registry = PluginRegistry::new(config).unwrap();
        
        let graph = registry.get_dependency_graph();
        assert!(graph.is_empty());
    }
}
