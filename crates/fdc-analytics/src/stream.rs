//! Stream processing engine

use crate::{config::StreamConfig, models::StreamEvent};
use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::{Stream, StreamExt};

/// 流处理器统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamStats {
    /// 处理的事件数
    pub events_processed: u64,
    /// 成功处理数
    pub events_success: u64,
    /// 失败处理数
    pub events_failed: u64,
    /// 平均处理时间（微秒）
    pub avg_processing_time_us: f64,
    /// 吞吐量（事件/秒）
    pub throughput_events_per_sec: f64,
}

impl StreamStats {
    /// 记录事件处理
    pub fn record_event(&mut self, success: bool, processing_time_us: u64) {
        self.events_processed += 1;
        
        if success {
            self.events_success += 1;
        } else {
            self.events_failed += 1;
        }
        
        // 更新平均处理时间
        self.avg_processing_time_us = 
            (self.avg_processing_time_us * (self.events_processed - 1) as f64 + processing_time_us as f64) 
            / self.events_processed as f64;
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.events_processed == 0 {
            0.0
        } else {
            self.events_success as f64 / self.events_processed as f64
        }
    }
}

/// 流处理器
pub struct StreamProcessor {
    /// 配置
    config: StreamConfig,
    /// 统计信息
    stats: Arc<RwLock<StreamStats>>,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl StreamProcessor {
    /// 创建新的流处理器
    pub fn new(config: StreamConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(StreamStats::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 处理事件流
    pub async fn process_stream<S, F, Fut>(&self, mut stream: S, processor: F) -> Result<()>
    where
        S: Stream<Item = StreamEvent> + Unpin,
        F: Fn(StreamEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);
        
        while let Some(event) = stream.next().await {
            if !*self.running.read().await {
                break;
            }
            
            let start_time = std::time::Instant::now();
            let result = processor(event).await;
            let processing_time_us = start_time.elapsed().as_micros() as u64;
            
            // 更新统计信息
            {
                let mut stats = self.stats.write().await;
                stats.record_event(result.is_ok(), processing_time_us);
            }
            
            if let Err(e) = result {
                tracing::error!("Stream processing error: {}", e);
                if !self.config.fault_tolerance {
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    /// 启动流处理器
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(Error::internal("Stream processor is already running"));
        }
        *running = true;
        
        tracing::info!("Stream processor started");
        Ok(())
    }
    
    /// 停止流处理器
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        
        tracing::info!("Stream processor stopped");
        Ok(())
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> StreamStats {
        self.stats.read().await.clone()
    }
    
    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = StreamStats::default();
    }
    
    /// 检查是否运行中
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

/// 流处理管道
pub struct StreamPipeline {
    /// 处理器列表
    processors: Vec<Arc<StreamProcessor>>,
    /// 管道配置
    config: StreamConfig,
}

impl StreamPipeline {
    /// 创建新的流处理管道
    pub fn new(config: StreamConfig) -> Self {
        Self {
            processors: Vec::new(),
            config,
        }
    }
    
    /// 添加处理器
    pub fn add_processor(&mut self, processor: Arc<StreamProcessor>) {
        self.processors.push(processor);
    }
    
    /// 启动管道
    pub async fn start(&self) -> Result<()> {
        for processor in &self.processors {
            processor.start().await?;
        }
        
        tracing::info!("Stream pipeline started with {} processors", self.processors.len());
        Ok(())
    }
    
    /// 停止管道
    pub async fn stop(&self) -> Result<()> {
        for processor in &self.processors {
            processor.stop().await?;
        }
        
        tracing::info!("Stream pipeline stopped");
        Ok(())
    }
    
    /// 获取管道统计信息
    pub async fn get_pipeline_stats(&self) -> Vec<StreamStats> {
        let mut stats = Vec::new();
        for processor in &self.processors {
            stats.push(processor.get_stats().await);
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::iter;

    #[test]
    fn test_stream_stats() {
        let mut stats = StreamStats::default();
        
        stats.record_event(true, 1000);
        stats.record_event(false, 2000);
        stats.record_event(true, 1500);
        
        assert_eq!(stats.events_processed, 3);
        assert_eq!(stats.events_success, 2);
        assert_eq!(stats.events_failed, 1);
        assert_eq!(stats.success_rate(), 2.0 / 3.0);
        assert_eq!(stats.avg_processing_time_us, 1500.0);
    }

    #[tokio::test]
    async fn test_stream_processor_lifecycle() {
        let config = StreamConfig::default();
        let processor = StreamProcessor::new(config);
        
        assert!(!processor.is_running().await);
        
        assert!(processor.start().await.is_ok());
        assert!(processor.is_running().await);
        
        assert!(processor.stop().await.is_ok());
        assert!(!processor.is_running().await);
    }

    #[tokio::test]
    async fn test_stream_processing() {
        let config = StreamConfig::default();
        let processor = StreamProcessor::new(config);
        
        // 创建测试事件流
        let events = vec![
            StreamEvent::new("test".to_string(), serde_json::json!({"value": 1})),
            StreamEvent::new("test".to_string(), serde_json::json!({"value": 2})),
        ];
        let stream = iter(events);
        
        // 简单的处理函数
        let process_fn = |_event: StreamEvent| async {
            Ok(())
        };
        
        // 处理流
        let result = processor.process_stream(stream, process_fn).await;
        assert!(result.is_ok());
        
        // 检查统计信息
        let stats = processor.get_stats().await;
        assert_eq!(stats.events_processed, 2);
        assert_eq!(stats.events_success, 2);
        assert_eq!(stats.events_failed, 0);
    }
}
