//! Error handling for Financial Data Center

use thiserror::Error;

/// 主要错误类型
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Type system error: {message}")]
    Type { message: String },
    
    #[error("WASM runtime error: {message}")]
    Wasm { message: String },
    
    #[error("Storage error: {message}")]
    Storage { message: String },
    
    #[error("Query error: {message}")]
    Query { message: String },
    
    #[error("Serialization error: {message}")]
    Serialization { message: String },
    
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Network error: {message}")]
    Network { message: String },
    
    #[error("Memory error: {message}")]
    Memory { message: String },
    
    #[error("Plugin error: {message}")]
    Plugin { message: String },
    
    #[error("Internal error: {message}")]
    Internal { message: String },
    
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    
    #[error("JSON error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },
    
    #[error("Bincode error: {source}")]
    Bincode {
        #[from]
        source: bincode::Error,
    },
    
    #[error("Decimal error: {source}")]
    Decimal {
        #[from]
        source: rust_decimal::Error,
    },
    
    #[error("UUID error: {source}")]
    Uuid {
        #[from]
        source: uuid::Error,
    },
    
    #[error("Parse error: {message}")]
    Parse { message: String },
    
    #[error("Timeout error: operation timed out after {duration_ms}ms")]
    Timeout { duration_ms: u64 },
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Already exists: {resource}")]
    AlreadyExists { resource: String },
    
    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },
    
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
    
    #[error("Invalid argument: {argument}")]
    InvalidArgument { argument: String },
    
    #[error("Unimplemented: {feature}")]
    Unimplemented { feature: String },
}

impl Error {
    /// 创建配置错误
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }
    
    /// 创建类型错误
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::Type {
            message: message.into(),
        }
    }
    
    /// 创建WASM错误
    pub fn wasm(message: impl Into<String>) -> Self {
        Self::Wasm {
            message: message.into(),
        }
    }
    
    /// 创建存储错误
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }
    
    /// 创建查询错误
    pub fn query(message: impl Into<String>) -> Self {
        Self::Query {
            message: message.into(),
        }
    }
    
    /// 创建序列化错误
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }
    
    /// 创建验证错误
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }
    
    /// 创建网络错误
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }
    
    /// 创建内存错误
    pub fn memory(message: impl Into<String>) -> Self {
        Self::Memory {
            message: message.into(),
        }
    }
    
    /// 创建插件错误
    pub fn plugin(message: impl Into<String>) -> Self {
        Self::Plugin {
            message: message.into(),
        }
    }
    
    /// 创建内部错误
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
    
    /// 创建解析错误
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }
    
    /// 创建超时错误
    pub fn timeout(duration_ms: u64) -> Self {
        Self::Timeout { duration_ms }
    }
    
    /// 创建未找到错误
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }
    
    /// 创建已存在错误
    pub fn already_exists(resource: impl Into<String>) -> Self {
        Self::AlreadyExists {
            resource: resource.into(),
        }
    }
    
    /// 创建权限拒绝错误
    pub fn permission_denied(operation: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
        }
    }
    
    /// 创建资源耗尽错误
    pub fn resource_exhausted(resource: impl Into<String>) -> Self {
        Self::ResourceExhausted {
            resource: resource.into(),
        }
    }
    
    /// 创建无效参数错误
    pub fn invalid_argument(argument: impl Into<String>) -> Self {
        Self::InvalidArgument {
            argument: argument.into(),
        }
    }
    
    /// 创建未实现错误
    pub fn unimplemented(feature: impl Into<String>) -> Self {
        Self::Unimplemented {
            feature: feature.into(),
        }
    }
    
    /// 检查是否为可重试错误
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Network { .. }
                | Error::Timeout { .. }
                | Error::ResourceExhausted { .. }
                | Error::Io { .. }
        )
    }
    
    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::Config { .. } => "CONFIG_ERROR",
            Error::Type { .. } => "TYPE_ERROR",
            Error::Wasm { .. } => "WASM_ERROR",
            Error::Storage { .. } => "STORAGE_ERROR",
            Error::Query { .. } => "QUERY_ERROR",
            Error::Serialization { .. } => "SERIALIZATION_ERROR",
            Error::Validation { .. } => "VALIDATION_ERROR",
            Error::Network { .. } => "NETWORK_ERROR",
            Error::Memory { .. } => "MEMORY_ERROR",
            Error::Plugin { .. } => "PLUGIN_ERROR",
            Error::Internal { .. } => "INTERNAL_ERROR",
            Error::Io { .. } => "IO_ERROR",
            Error::Json { .. } => "JSON_ERROR",
            Error::Bincode { .. } => "BINCODE_ERROR",
            Error::Decimal { .. } => "DECIMAL_ERROR",
            Error::Uuid { .. } => "UUID_ERROR",
            Error::Parse { .. } => "PARSE_ERROR",
            Error::Timeout { .. } => "TIMEOUT_ERROR",
            Error::NotFound { .. } => "NOT_FOUND",
            Error::AlreadyExists { .. } => "ALREADY_EXISTS",
            Error::PermissionDenied { .. } => "PERMISSION_DENIED",
            Error::ResourceExhausted { .. } => "RESOURCE_EXHAUSTED",
            Error::InvalidArgument { .. } => "INVALID_ARGUMENT",
            Error::Unimplemented { .. } => "UNIMPLEMENTED",
        }
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// 错误上下文扩展
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<Error>,
{
    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| {
            let original_error = e.into();
            Error::internal(format!("{}: {}", context, original_error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::config("test config error");
        assert_eq!(err.error_code(), "CONFIG_ERROR");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_retryable_errors() {
        let network_err = Error::network("connection failed");
        assert!(network_err.is_retryable());
        
        let timeout_err = Error::timeout(5000);
        assert!(timeout_err.is_retryable());
        
        let config_err = Error::config("invalid config");
        assert!(!config_err.is_retryable());
    }

    #[test]
    fn test_error_context() {
        let result: std::result::Result<(), std::io::Error> = 
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        
        let with_context = result.with_context("loading configuration");
        assert!(with_context.is_err());
    }
}
