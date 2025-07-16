//! Data buffering and flow control

use crate::config::BufferConfig;
use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// 缓冲区统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BufferStats {
    /// 当前缓冲区大小
    pub current_size: usize,
    /// 最大缓冲区大小
    pub max_size: usize,
    /// 缓冲区使用率
    pub utilization: f64,
    /// 入队消息数
    pub enqueued_messages: u64,
    /// 出队消息数
    pub dequeued_messages: u64,
    /// 丢弃消息数
    pub dropped_messages: u64,
}

/// 数据缓冲区
pub struct DataBuffer<T> {
    /// 配置
    config: BufferConfig,
    /// 内部缓冲区
    buffer: Arc<Mutex<VecDeque<T>>>,
    /// 统计信息
    stats: Arc<RwLock<BufferStats>>,
}

impl<T> DataBuffer<T> {
    /// 创建新的数据缓冲区
    pub fn new(config: BufferConfig) -> Self {
        Self {
            config: config.clone(),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(BufferStats {
                max_size: config.buffer_size,
                ..Default::default()
            })),
        }
    }
    
    /// 入队数据
    pub async fn enqueue(&self, item: T) -> Result<()> {
        let mut buffer = self.buffer.lock().await;
        let mut stats = self.stats.write().await;
        
        // 检查缓冲区是否已满
        if buffer.len() >= self.config.buffer_size {
            stats.dropped_messages += 1;
            return Err(Error::resource_exhausted("Buffer is full"));
        }
        
        buffer.push_back(item);
        stats.enqueued_messages += 1;
        stats.current_size = buffer.len();
        stats.utilization = stats.current_size as f64 / stats.max_size as f64;
        
        Ok(())
    }
    
    /// 出队数据
    pub async fn dequeue(&self) -> Option<T> {
        let mut buffer = self.buffer.lock().await;
        let mut stats = self.stats.write().await;
        
        if let Some(item) = buffer.pop_front() {
            stats.dequeued_messages += 1;
            stats.current_size = buffer.len();
            stats.utilization = stats.current_size as f64 / stats.max_size as f64;
            Some(item)
        } else {
            None
        }
    }
    
    /// 获取缓冲区大小
    pub async fn size(&self) -> usize {
        self.buffer.lock().await.len()
    }
    
    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        self.buffer.lock().await.is_empty()
    }
    
    /// 检查是否已满
    pub async fn is_full(&self) -> bool {
        self.buffer.lock().await.len() >= self.config.buffer_size
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> BufferStats {
        self.stats.read().await.clone()
    }
    
    /// 清空缓冲区
    pub async fn clear(&self) {
        let mut buffer = self.buffer.lock().await;
        let mut stats = self.stats.write().await;
        
        buffer.clear();
        stats.current_size = 0;
        stats.utilization = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buffer_operations() {
        let config = BufferConfig {
            buffer_size: 3,
            ..Default::default()
        };
        let buffer = DataBuffer::new(config);
        
        // 测试入队
        assert!(buffer.enqueue("item1".to_string()).await.is_ok());
        assert!(buffer.enqueue("item2".to_string()).await.is_ok());
        assert!(buffer.enqueue("item3".to_string()).await.is_ok());
        
        // 缓冲区已满
        assert!(buffer.enqueue("item4".to_string()).await.is_err());
        assert!(buffer.is_full().await);
        
        // 测试出队
        assert_eq!(buffer.dequeue().await, Some("item1".to_string()));
        assert_eq!(buffer.dequeue().await, Some("item2".to_string()));
        assert_eq!(buffer.size().await, 1);
        
        // 清空缓冲区
        buffer.clear().await;
        assert!(buffer.is_empty().await);
    }
}
