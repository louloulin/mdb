//! High-performance network data receiver

use crate::config::ReceiverConfig;
use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_util::codec::{Framed, LinesCodec};
use futures::StreamExt;
use tracing::{debug, error, info, warn};

/// 接收器类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReceiverType {
    /// TCP接收器
    Tcp,
    /// UDP接收器
    Udp,
    /// WebSocket接收器
    WebSocket,
    /// QUIC接收器
    Quic,
    /// 自定义接收器
    Custom(String),
}

impl std::fmt::Display for ReceiverType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiverType::Tcp => write!(f, "tcp"),
            ReceiverType::Udp => write!(f, "udp"),
            ReceiverType::WebSocket => write!(f, "websocket"),
            ReceiverType::Quic => write!(f, "quic"),
            ReceiverType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

/// 接收到的数据
#[derive(Debug, Clone)]
pub struct ReceivedData {
    /// 数据内容
    pub data: Vec<u8>,
    /// 来源地址
    pub source: SocketAddr,
    /// 接收时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 数据大小
    pub size: usize,
    /// 连接ID
    pub connection_id: String,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl ReceivedData {
    /// 创建新的接收数据
    pub fn new(data: Vec<u8>, source: SocketAddr, connection_id: String) -> Self {
        let size = data.len();
        Self {
            data,
            source,
            timestamp: chrono::Utc::now(),
            size,
            connection_id,
            metadata: HashMap::new(),
        }
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// 获取数据作为字符串
    pub fn as_string(&self) -> Result<String> {
        String::from_utf8(self.data.clone())
            .map_err(|e| Error::validation(format!("Invalid UTF-8 data: {}", e)))
    }
}

/// 连接统计信息
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// 接收的消息数
    pub messages_received: u64,
    /// 接收的字节数
    pub bytes_received: u64,
    /// 连接建立时间
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 最后活动时间
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    /// 错误计数
    pub error_count: u64,
}

/// 数据接收器
pub struct DataReceiver {
    /// 配置
    config: ReceiverConfig,
    /// 接收器类型
    receiver_type: ReceiverType,
    /// 数据发送通道
    data_sender: mpsc::UnboundedSender<ReceivedData>,
    /// 连接统计
    connection_stats: Arc<RwLock<HashMap<String, ConnectionStats>>>,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl DataReceiver {
    /// 创建新的数据接收器
    pub fn new(
        config: ReceiverConfig,
        receiver_type: ReceiverType,
        data_sender: mpsc::UnboundedSender<ReceivedData>,
    ) -> Self {
        Self {
            config,
            receiver_type,
            data_sender,
            connection_stats: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 启动接收器
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(Error::internal("Receiver is already running"));
        }
        *running = true;
        drop(running);
        
        match self.receiver_type {
            ReceiverType::Tcp => self.start_tcp_receiver().await,
            ReceiverType::WebSocket => self.start_websocket_receiver().await,
            _ => Err(Error::unimplemented(format!("Receiver type: {}", self.receiver_type))),
        }
    }
    
    /// 停止接收器
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("Data receiver stopped");
        Ok(())
    }
    
    /// 启动TCP接收器
    async fn start_tcp_receiver(&self) -> Result<()> {
        let bind_addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TcpListener::bind(&bind_addr).await
            .map_err(|e| Error::io(e))?;
        
        info!("TCP receiver listening on {}", bind_addr);
        
        let data_sender = self.data_sender.clone();
        let connection_stats = self.connection_stats.clone();
        let running = self.running.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            while *running.read().await {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let connection_id = uuid::Uuid::new_v4().to_string();
                        debug!("New TCP connection: {} from {}", connection_id, addr);
                        
                        // 更新连接统计
                        {
                            let mut stats = connection_stats.write().await;
                            stats.insert(connection_id.clone(), ConnectionStats {
                                connected_at: Some(chrono::Utc::now()),
                                ..Default::default()
                            });
                        }
                        
                        let sender = data_sender.clone();
                        let stats = connection_stats.clone();
                        let conn_config = config.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_tcp_connection(
                                stream, addr, connection_id.clone(), sender, stats, conn_config
                            ).await {
                                error!("Error handling TCP connection {}: {}", connection_id, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept TCP connection: {}", e);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// 处理TCP连接
    async fn handle_tcp_connection(
        stream: TcpStream,
        addr: SocketAddr,
        connection_id: String,
        data_sender: mpsc::UnboundedSender<ReceivedData>,
        connection_stats: Arc<RwLock<HashMap<String, ConnectionStats>>>,
        config: ReceiverConfig,
    ) -> Result<()> {
        // 设置TCP选项
        if config.tcp_nodelay {
            stream.set_nodelay(true).map_err(|e| Error::io(e))?;
        }
        
        let mut framed = Framed::new(stream, LinesCodec::new());
        
        while let Some(result) = framed.next().await {
            match result {
                Ok(line) => {
                    let data = line.into_bytes();
                    let size = data.len();
                    
                    let received_data = ReceivedData::new(data, addr, connection_id.clone());
                    
                    if let Err(e) = data_sender.send(received_data) {
                        error!("Failed to send received data: {}", e);
                        break;
                    }
                    
                    // 更新统计信息
                    {
                        let mut stats = connection_stats.write().await;
                        if let Some(conn_stats) = stats.get_mut(&connection_id) {
                            conn_stats.messages_received += 1;
                            conn_stats.bytes_received += size as u64;
                            conn_stats.last_activity = Some(chrono::Utc::now());
                        }
                    }
                }
                Err(e) => {
                    warn!("Error reading from TCP connection {}: {}", connection_id, e);
                    
                    // 更新错误统计
                    {
                        let mut stats = connection_stats.write().await;
                        if let Some(conn_stats) = stats.get_mut(&connection_id) {
                            conn_stats.error_count += 1;
                        }
                    }
                    break;
                }
            }
        }
        
        // 清理连接统计
        {
            let mut stats = connection_stats.write().await;
            stats.remove(&connection_id);
        }
        
        debug!("TCP connection {} closed", connection_id);
        Ok(())
    }
    
    /// 启动WebSocket接收器
    async fn start_websocket_receiver(&self) -> Result<()> {
        // TODO: 实现WebSocket接收器
        Err(Error::unimplemented("WebSocket receiver not implemented"))
    }
    
    /// 获取连接统计信息
    pub async fn get_connection_stats(&self) -> HashMap<String, ConnectionStats> {
        self.connection_stats.read().await.clone()
    }
    
    /// 获取活跃连接数
    pub async fn get_active_connections(&self) -> usize {
        self.connection_stats.read().await.len()
    }
    
    /// 检查是否运行中
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[test]
    fn test_receiver_type_display() {
        assert_eq!(ReceiverType::Tcp.to_string(), "tcp");
        assert_eq!(ReceiverType::WebSocket.to_string(), "websocket");
        assert_eq!(ReceiverType::Custom("test".to_string()).to_string(), "custom_test");
    }

    #[test]
    fn test_received_data_creation() {
        let data = b"test data".to_vec();
        let addr = "127.0.0.1:8080".parse().unwrap();
        let conn_id = "test-conn".to_string();
        
        let received = ReceivedData::new(data.clone(), addr, conn_id.clone());
        
        assert_eq!(received.data, data);
        assert_eq!(received.source, addr);
        assert_eq!(received.connection_id, conn_id);
        assert_eq!(received.size, data.len());
        assert!(received.metadata.is_empty());
    }

    #[tokio::test]
    async fn test_receiver_lifecycle() {
        let (sender, _receiver) = mpsc::unbounded_channel();
        let config = ReceiverConfig::default();
        let receiver = DataReceiver::new(config, ReceiverType::Tcp, sender);
        
        assert!(!receiver.is_running().await);
        assert_eq!(receiver.get_active_connections().await, 0);
        
        // 测试停止未运行的接收器
        assert!(receiver.stop().await.is_ok());
    }
}
