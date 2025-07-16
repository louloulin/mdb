//! # Financial Data Center API Layer
//!
//! This crate provides comprehensive API interfaces for the Financial Data Center,
//! including REST, gRPC, GraphQL, and WebSocket APIs for data access and management.

pub mod rest;           // REST API实现
pub mod grpc;           // gRPC API实现
pub mod graphql;        // GraphQL API实现
pub mod websocket;      // WebSocket API实现
pub mod auth;           // 认证和授权
pub mod middleware;     // 中间件
pub mod config;         // API配置
pub mod server;         // 服务器管理
pub mod handlers;       // 请求处理器
pub mod models;         // API数据模型
pub mod errors;         // API错误处理
pub mod metrics;        // API指标

// 重新导出常用类型
pub use server::{ApiServer, ServerConfig};
pub use config::ApiConfig;
pub use errors::{ApiError, ApiResult};
pub use models::{ApiResponse, QueryRequest, QueryResponse};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 默认REST API端口
pub const DEFAULT_REST_PORT: u16 = 8080;

/// 默认gRPC端口
pub const DEFAULT_GRPC_PORT: u16 = 9090;

/// 默认GraphQL端点
pub const DEFAULT_GRAPHQL_ENDPOINT: &str = "/graphql";

/// 默认WebSocket端点
pub const DEFAULT_WEBSOCKET_ENDPOINT: &str = "/ws";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "fdc-api");
        assert_eq!(DEFAULT_REST_PORT, 8080);
        assert_eq!(DEFAULT_GRPC_PORT, 9090);
        assert_eq!(DEFAULT_GRAPHQL_ENDPOINT, "/graphql");
        assert_eq!(DEFAULT_WEBSOCKET_ENDPOINT, "/ws");
    }
}
