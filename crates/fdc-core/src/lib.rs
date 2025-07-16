//! # Financial Data Center Core Library
//!
//! This crate provides core types, utilities, and abstractions for the
//! Financial Data Center high-performance trading database system.

pub mod types;          // 核心数据类型
pub mod config;         // 配置管理
pub mod error;          // 错误处理
pub mod metrics;        // 性能指标
pub mod time;           // 时间处理
pub mod memory;         // 内存管理
pub mod wasm_bridge;    // WASM桥接
pub mod type_registry;  // 类型注册表

// 重新导出常用类型
pub use types::*;
pub use error::{Error, Result};
pub use config::Config;
pub use metrics::Metrics;

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "fdc-core");
    }
}
