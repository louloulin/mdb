//! # Financial Data Center WASM Plugin System
//!
//! This crate provides a comprehensive WASM plugin system for the
//! Financial Data Center, enabling hot-loadable, multi-language plugins
//! with security sandboxing and high performance.

pub mod runtime;        // WASM运行时
pub mod plugin;         // 插件管理
pub mod registry;       // 插件注册表
pub mod security;       // 安全沙箱
pub mod loader;         // 插件加载器
pub mod bridge;         // 主机-WASM桥接
pub mod types;          // WASM类型定义
pub mod events;         // 事件系统
pub mod metrics;        // 指标收集

// 重新导出常用类型
pub use runtime::{WasmRuntime, WasmRuntimeConfig};
pub use plugin::{WasmPlugin, PluginInfo, PluginType};
pub use registry::PluginRegistry;
pub use security::{SecurityPolicy, SecurityViolation};
pub use loader::PluginLoader;
pub use bridge::{WasmBridge, HostFunction};
pub use types::{WasmValue, WasmType};
pub use events::{WasmEvent, WasmEventListener};
pub use metrics::WasmMetrics;

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 默认WASM内存限制 (128MB)
pub const DEFAULT_MEMORY_LIMIT: usize = 128 * 1024 * 1024;

/// 默认执行超时 (5秒)
pub const DEFAULT_EXECUTION_TIMEOUT_MS: u64 = 5000;

/// 默认最大插件数量
pub const DEFAULT_MAX_PLUGINS: usize = 100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "fdc-wasm");
        assert_eq!(DEFAULT_MEMORY_LIMIT, 128 * 1024 * 1024);
        assert_eq!(DEFAULT_EXECUTION_TIMEOUT_MS, 5000);
        assert_eq!(DEFAULT_MAX_PLUGINS, 100);
    }
}
