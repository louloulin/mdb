//! Protocol support for various data formats

use serde::{Deserialize, Serialize};

/// 支持的协议类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolType {
    /// TCP协议
    Tcp,
    /// UDP协议
    Udp,
    /// WebSocket协议
    WebSocket,
    /// QUIC协议
    Quic,
    /// HTTP协议
    Http,
    /// FIX协议
    Fix,
    /// 自定义协议
    Custom(String),
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolType::Tcp => write!(f, "tcp"),
            ProtocolType::Udp => write!(f, "udp"),
            ProtocolType::WebSocket => write!(f, "websocket"),
            ProtocolType::Quic => write!(f, "quic"),
            ProtocolType::Http => write!(f, "http"),
            ProtocolType::Fix => write!(f, "fix"),
            ProtocolType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

/// 协议处理器特征
pub trait ProtocolHandler {
    /// 处理协议数据
    fn handle_data(&self, data: &[u8]) -> Result<Vec<u8>, String>;
    
    /// 获取协议类型
    fn protocol_type(&self) -> ProtocolType;
    
    /// 验证协议数据
    fn validate_data(&self, data: &[u8]) -> bool;
}

/// FIX协议处理器
pub struct FixProtocolHandler;

impl ProtocolHandler for FixProtocolHandler {
    fn handle_data(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // 简化的FIX协议处理
        Ok(data.to_vec())
    }
    
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Fix
    }
    
    fn validate_data(&self, data: &[u8]) -> bool {
        // 简化的FIX验证：检查是否包含SOH分隔符
        data.contains(&0x01)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_type_display() {
        assert_eq!(ProtocolType::Tcp.to_string(), "tcp");
        assert_eq!(ProtocolType::Fix.to_string(), "fix");
        assert_eq!(ProtocolType::Custom("test".to_string()).to_string(), "custom_test");
    }

    #[test]
    fn test_fix_protocol_handler() {
        let handler = FixProtocolHandler;
        assert_eq!(handler.protocol_type(), ProtocolType::Fix);
        
        let fix_data = b"8=FIX.4.2\x019=40\x0135=D\x01";
        assert!(handler.validate_data(fix_data));
        
        let non_fix_data = b"regular data";
        assert!(!handler.validate_data(non_fix_data));
    }
}
