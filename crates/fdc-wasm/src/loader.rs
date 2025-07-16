//! WASM plugin loader

use crate::{
    plugin::{WasmPlugin, PluginInfo, PluginType},
    security::SecurityPolicy,
};
use fdc_core::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
// use notify::{Watcher, RecursiveMode, RecommendedWatcher, Event, EventKind};
// use std::sync::mpsc;
// use std::time::Duration;

/// 插件配置文件格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
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
    /// WASM文件路径（相对于配置文件）
    pub wasm_file: String,
    /// 依赖项
    pub dependencies: Vec<String>,
    /// 导出的函数
    pub exported_functions: Vec<String>,
    /// 权限要求
    pub permissions: Vec<String>,
    /// 安全策略覆盖
    pub security_policy: Option<SecurityPolicyConfig>,
    /// 配置参数
    pub config: Option<std::collections::HashMap<String, String>>,
    /// 元数据
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// 安全策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicyConfig {
    pub memory_limit: Option<usize>,
    pub execution_timeout_ms: Option<u64>,
    pub network_access: Option<bool>,
    pub file_access: Option<bool>,
    pub env_access: Option<bool>,
    pub allowed_paths: Option<Vec<String>>,
    pub allowed_hosts: Option<Vec<String>>,
}

impl SecurityPolicyConfig {
    /// 应用到安全策略
    pub fn apply_to(&self, policy: &mut SecurityPolicy) {
        if let Some(memory_limit) = self.memory_limit {
            policy.memory_limit = memory_limit;
        }
        if let Some(execution_timeout_ms) = self.execution_timeout_ms {
            policy.execution_timeout_ms = execution_timeout_ms;
        }
        if let Some(network_access) = self.network_access {
            policy.network_access = network_access;
        }
        if let Some(file_access) = self.file_access {
            policy.file_access = file_access;
        }
        if let Some(env_access) = self.env_access {
            policy.env_access = env_access;
        }
        if let Some(ref allowed_paths) = self.allowed_paths {
            policy.allowed_paths = allowed_paths.iter().cloned().collect();
        }
        if let Some(ref allowed_hosts) = self.allowed_hosts {
            policy.allowed_hosts = allowed_hosts.iter().cloned().collect();
        }
    }
}

/// 插件加载器
pub struct PluginLoader {
    /// 插件目录
    plugin_dir: PathBuf,
    /// 默认安全策略
    default_security_policy: SecurityPolicy,
    /// 是否启用热加载
    hot_reload_enabled: bool,
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new(plugin_dir: PathBuf, default_security_policy: SecurityPolicy) -> Self {
        Self {
            plugin_dir,
            default_security_policy,
            hot_reload_enabled: false,
        }
    }

    /// 启用热加载（暂未实现）
    pub fn enable_hot_reload(&mut self) -> Result<()> {
        // TODO: 实现热加载功能
        self.hot_reload_enabled = true;
        Ok(())
    }

    /// 禁用热加载
    pub fn disable_hot_reload(&mut self) {
        self.hot_reload_enabled = false;
    }
    
    /// 扫描插件目录
    pub fn scan_plugins(&self) -> Result<Vec<PluginConfig>> {
        let mut configs = Vec::new();
        
        if !self.plugin_dir.exists() {
            return Ok(configs);
        }
        
        for entry in fs::read_dir(&self.plugin_dir)
            .map_err(|e| Error::io(e))?
        {
            let entry = entry.map_err(|e| Error::io(e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // 查找插件配置文件
                let config_file = path.join("plugin.toml");
                if config_file.exists() {
                    match self.load_plugin_config(&config_file) {
                        Ok(config) => configs.push(config),
                        Err(e) => {
                            tracing::warn!("Failed to load plugin config from {:?}: {}", config_file, e);
                        }
                    }
                }
            }
        }
        
        Ok(configs)
    }
    
    /// 加载插件配置
    pub fn load_plugin_config(&self, config_path: &Path) -> Result<PluginConfig> {
        let content = fs::read_to_string(config_path)
            .map_err(|e| Error::io(e))?;
        
        let config: PluginConfig = toml::from_str(&content)
            .map_err(|e| Error::config(format!("Failed to parse plugin config: {}", e)))?;
        
        Ok(config)
    }
    
    /// 从配置加载插件
    pub fn load_plugin_from_config(&self, config: &PluginConfig, config_dir: &Path) -> Result<WasmPlugin> {
        // 构建WASM文件路径
        let wasm_path = config_dir.join(&config.wasm_file);
        if !wasm_path.exists() {
            return Err(Error::not_found(format!("WASM file: {:?}", wasm_path)));
        }
        
        // 读取WASM字节码
        let wasm_bytes = fs::read(&wasm_path)
            .map_err(|e| Error::io(e))?;
        
        // 创建插件信息
        let mut plugin_info = PluginInfo::new(
            config.name.clone(),
            config.version.clone(),
            config.plugin_type.clone(),
            wasm_path,
        );
        
        // 设置可选字段
        plugin_info.description = config.description.clone();
        plugin_info.author = config.author.clone();
        plugin_info.dependencies = config.dependencies.clone();
        plugin_info.exported_functions = config.exported_functions.clone();
        plugin_info.permissions = config.permissions.clone();
        
        if let Some(ref metadata) = config.metadata {
            for (key, value) in metadata {
                plugin_info.set_metadata(key.clone(), value.clone());
            }
        }
        
        // 创建安全策略
        let mut security_policy = self.default_security_policy.clone();
        if let Some(ref policy_config) = config.security_policy {
            policy_config.apply_to(&mut security_policy);
        }
        
        // 验证安全策略
        security_policy.validate()?;
        
        // 创建插件
        let mut plugin = WasmPlugin::new(plugin_info, wasm_bytes, security_policy);
        
        // 设置配置参数
        if let Some(ref plugin_config) = config.config {
            for (key, value) in plugin_config {
                plugin.set_config(key.clone(), value.clone());
            }
        }
        
        // 验证插件
        plugin.validate()?;
        
        Ok(plugin)
    }
    
    /// 从目录加载插件
    pub fn load_plugin_from_dir(&self, plugin_dir: &Path) -> Result<WasmPlugin> {
        let config_file = plugin_dir.join("plugin.toml");
        let config = self.load_plugin_config(&config_file)?;
        self.load_plugin_from_config(&config, plugin_dir)
    }
    
    /// 加载所有插件
    pub fn load_all_plugins(&self) -> Result<Vec<WasmPlugin>> {
        let configs = self.scan_plugins()?;
        let mut plugins = Vec::new();
        
        for config in configs {
            let plugin_dir = self.plugin_dir.join(&config.name);
            match self.load_plugin_from_config(&config, &plugin_dir) {
                Ok(plugin) => plugins.push(plugin),
                Err(e) => {
                    tracing::error!("Failed to load plugin {}: {}", config.name, e);
                }
            }
        }
        
        Ok(plugins)
    }
    
    /// 检查文件更改（暂未实现）
    pub fn check_for_changes(&self) -> Vec<PathBuf> {
        // TODO: 实现文件更改检测
        Vec::new()
    }
    
    /// 验证插件文件
    pub fn validate_plugin_file(&self, wasm_path: &Path) -> Result<()> {
        if !wasm_path.exists() {
            return Err(Error::not_found(format!("WASM file: {:?}", wasm_path)));
        }
        
        let wasm_bytes = fs::read(wasm_path)
            .map_err(|e| Error::io(e))?;
        
        // 检查WASM魔数
        if wasm_bytes.len() < 4 || &wasm_bytes[0..4] != b"\0asm" {
            return Err(Error::validation("Invalid WASM file: missing magic number"));
        }
        
        // 检查版本
        if wasm_bytes.len() < 8 {
            return Err(Error::validation("Invalid WASM file: missing version"));
        }
        
        let version = u32::from_le_bytes([
            wasm_bytes[4], wasm_bytes[5], wasm_bytes[6], wasm_bytes[7]
        ]);
        
        if version != 1 {
            return Err(Error::validation(format!("Unsupported WASM version: {}", version)));
        }
        
        Ok(())
    }
    
    /// 创建插件模板
    pub fn create_plugin_template(&self, plugin_name: &str, plugin_type: PluginType) -> Result<()> {
        let plugin_dir = self.plugin_dir.join(plugin_name);
        fs::create_dir_all(&plugin_dir)
            .map_err(|e| Error::io(e))?;
        
        // 创建配置文件
        let config = PluginConfig {
            name: plugin_name.to_string(),
            version: "0.1.0".to_string(),
            description: Some(format!("A {} plugin", plugin_type)),
            author: None,
            plugin_type,
            wasm_file: format!("{}.wasm", plugin_name),
            dependencies: Vec::new(),
            exported_functions: vec!["process".to_string()],
            permissions: Vec::new(),
            security_policy: None,
            config: None,
            metadata: None,
        };
        
        let config_content = toml::to_string_pretty(&config)
            .map_err(|e| Error::serialization(e.to_string()))?;
        
        let config_file = plugin_dir.join("plugin.toml");
        fs::write(config_file, config_content)
            .map_err(|e| Error::io(e))?;
        
        // 创建README文件
        let readme_content = format!(
            "# {}\n\n{}\n\n## Building\n\n```bash\n# Build the WASM module\ncargo build --target wasm32-unknown-unknown --release\n```\n",
            plugin_name,
            config.description.as_deref().unwrap_or("Plugin description")
        );
        
        let readme_file = plugin_dir.join("README.md");
        fs::write(readme_file, readme_content)
            .map_err(|e| Error::io(e))?;
        
        Ok(())
    }
    
    /// 获取插件目录
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }
    
    /// 检查是否启用热加载
    pub fn is_hot_reload_enabled(&self) -> bool {
        self.hot_reload_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_loader_creation() {
        let temp_dir = TempDir::new().unwrap();
        let security_policy = SecurityPolicy::default();
        
        let loader = PluginLoader::new(temp_dir.path().to_path_buf(), security_policy);
        assert_eq!(loader.plugin_dir(), temp_dir.path());
        assert!(!loader.is_hot_reload_enabled());
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let security_policy = SecurityPolicy::default();
        
        let loader = PluginLoader::new(temp_dir.path().to_path_buf(), security_policy);
        let configs = loader.scan_plugins().unwrap();
        assert!(configs.is_empty());
    }

    #[test]
    fn test_create_plugin_template() {
        let temp_dir = TempDir::new().unwrap();
        let security_policy = SecurityPolicy::default();
        
        let loader = PluginLoader::new(temp_dir.path().to_path_buf(), security_policy);
        let result = loader.create_plugin_template("test_plugin", PluginType::DataTransform);
        assert!(result.is_ok());
        
        let plugin_dir = temp_dir.path().join("test_plugin");
        assert!(plugin_dir.exists());
        assert!(plugin_dir.join("plugin.toml").exists());
        assert!(plugin_dir.join("README.md").exists());
    }

    #[test]
    fn test_validate_plugin_file() {
        let temp_dir = TempDir::new().unwrap();
        let security_policy = SecurityPolicy::default();
        
        let loader = PluginLoader::new(temp_dir.path().to_path_buf(), security_policy);
        
        // 测试不存在的文件
        let nonexistent_file = temp_dir.path().join("nonexistent.wasm");
        assert!(loader.validate_plugin_file(&nonexistent_file).is_err());
        
        // 测试无效的WASM文件
        let invalid_wasm = temp_dir.path().join("invalid.wasm");
        fs::write(&invalid_wasm, b"invalid").unwrap();
        assert!(loader.validate_plugin_file(&invalid_wasm).is_err());
        
        // 测试有效的WASM文件（简化版）
        let valid_wasm = temp_dir.path().join("valid.wasm");
        let mut wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d]; // WASM魔数
        wasm_bytes.extend_from_slice(&1u32.to_le_bytes()); // 版本1
        fs::write(&valid_wasm, wasm_bytes).unwrap();
        assert!(loader.validate_plugin_file(&valid_wasm).is_ok());
    }
}
