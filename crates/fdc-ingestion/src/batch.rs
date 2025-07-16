//! Batch processing for high-throughput data ingestion

use crate::{config::BatchConfig, parser::ParsedData, validator::ValidationResult};
use fdc_core::error::{Error, Result};
// use fdc_storage::engine::StorageEngine; // 暂时注释掉

/// 简化的存储接口
#[async_trait::async_trait]
pub trait SimpleStorage: Send + Sync {
    async fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;
}
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};

/// 批量处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// 批量ID
    pub batch_id: String,
    /// 处理的消息数
    pub processed_count: usize,
    /// 成功处理数
    pub success_count: usize,
    /// 失败处理数
    pub failure_count: usize,
    /// 批量大小
    pub batch_size: usize,
    /// 处理耗时（毫秒）
    pub processing_time_ms: u64,
    /// 处理时间戳
    pub processed_at: chrono::DateTime<chrono::Utc>,
    /// 错误信息
    pub errors: Vec<String>,
}

impl BatchResult {
    /// 创建新的批量结果
    pub fn new(batch_id: String, batch_size: usize, processing_time_ms: u64) -> Self {
        Self {
            batch_id,
            processed_count: 0,
            success_count: 0,
            failure_count: 0,
            batch_size,
            processing_time_ms,
            processed_at: chrono::Utc::now(),
            errors: Vec::new(),
        }
    }
    
    /// 记录成功处理
    pub fn record_success(&mut self) {
        self.processed_count += 1;
        self.success_count += 1;
    }
    
    /// 记录失败处理
    pub fn record_failure(&mut self, error: String) {
        self.processed_count += 1;
        self.failure_count += 1;
        self.errors.push(error);
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.processed_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.processed_count as f64
        }
    }
    
    /// 获取处理吞吐量（消息/秒）
    pub fn throughput(&self) -> f64 {
        if self.processing_time_ms == 0 {
            0.0
        } else {
            (self.processed_count as f64 * 1000.0) / self.processing_time_ms as f64
        }
    }
}

/// 批量数据项
#[derive(Debug, Clone)]
pub struct BatchItem {
    /// 解析后的数据
    pub parsed_data: ParsedData,
    /// 验证结果
    pub validation_result: ValidationResult,
    /// 项目ID
    pub item_id: String,
}

impl BatchItem {
    /// 创建新的批量项
    pub fn new(parsed_data: ParsedData, validation_result: ValidationResult) -> Self {
        Self {
            parsed_data,
            validation_result,
            item_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    
    /// 检查是否有效
    pub fn is_valid(&self) -> bool {
        self.validation_result.is_valid
    }
}

/// 批量处理器统计信息
#[derive(Debug, Clone, Default)]
pub struct BatchProcessorStats {
    /// 处理的批次数
    pub batches_processed: u64,
    /// 处理的消息总数
    pub total_messages: u64,
    /// 成功处理的消息数
    pub successful_messages: u64,
    /// 失败处理的消息数
    pub failed_messages: u64,
    /// 总处理时间（毫秒）
    pub total_processing_time_ms: u64,
    /// 平均批量大小
    pub avg_batch_size: f64,
    /// 平均处理时间（毫秒）
    pub avg_processing_time_ms: f64,
    /// 吞吐量（消息/秒）
    pub throughput_msg_per_sec: f64,
}

impl BatchProcessorStats {
    /// 记录批量处理结果
    pub fn record_batch(&mut self, result: &BatchResult) {
        self.batches_processed += 1;
        self.total_messages += result.processed_count as u64;
        self.successful_messages += result.success_count as u64;
        self.failed_messages += result.failure_count as u64;
        self.total_processing_time_ms += result.processing_time_ms;
        
        // 更新平均值
        self.avg_batch_size = self.total_messages as f64 / self.batches_processed as f64;
        self.avg_processing_time_ms = self.total_processing_time_ms as f64 / self.batches_processed as f64;
        
        // 计算吞吐量
        if self.total_processing_time_ms > 0 {
            self.throughput_msg_per_sec = (self.total_messages as f64 * 1000.0) / self.total_processing_time_ms as f64;
        }
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_messages == 0 {
            0.0
        } else {
            self.successful_messages as f64 / self.total_messages as f64
        }
    }
}

/// 批量处理器
pub struct BatchProcessor {
    /// 配置
    config: BatchConfig,
    /// 存储引擎
    storage_engine: Arc<dyn SimpleStorage>,
    /// 统计信息
    stats: Arc<RwLock<BatchProcessorStats>>,
    /// 当前批次
    current_batch: Arc<RwLock<Vec<BatchItem>>>,
    /// 批次超时定时器
    batch_timer: Arc<RwLock<Option<Instant>>>,
}

impl BatchProcessor {
    /// 创建新的批量处理器
    pub fn new(config: BatchConfig, storage_engine: Arc<dyn SimpleStorage>) -> Self {
        Self {
            config,
            storage_engine,
            stats: Arc::new(RwLock::new(BatchProcessorStats::default())),
            current_batch: Arc::new(RwLock::new(Vec::new())),
            batch_timer: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 添加项目到批次
    pub async fn add_item(&self, item: BatchItem) -> Result<Option<BatchResult>> {
        let mut batch = self.current_batch.write().await;
        let mut timer = self.batch_timer.write().await;
        
        // 如果是第一个项目，启动定时器
        if batch.is_empty() {
            *timer = Some(Instant::now());
        }
        
        batch.push(item);
        
        // 检查是否需要处理批次
        let should_process = batch.len() >= self.config.batch_size
            || (timer.is_some() && timer.unwrap().elapsed() >= self.config.batch_timeout);
        
        if should_process {
            let items = batch.drain(..).collect();
            *timer = None;
            drop(batch);
            drop(timer);
            
            Ok(Some(self.process_batch(items).await?))
        } else {
            Ok(None)
        }
    }
    
    /// 强制处理当前批次
    pub async fn flush(&self) -> Result<Option<BatchResult>> {
        let mut batch = self.current_batch.write().await;
        let mut timer = self.batch_timer.write().await;
        
        if batch.is_empty() {
            return Ok(None);
        }
        
        let items = batch.drain(..).collect();
        *timer = None;
        drop(batch);
        drop(timer);
        
        Ok(Some(self.process_batch(items).await?))
    }
    
    /// 处理批次
    async fn process_batch(&self, items: Vec<BatchItem>) -> Result<BatchResult> {
        let batch_id = uuid::Uuid::new_v4().to_string();
        let batch_size = items.len();
        let start_time = Instant::now();
        
        let mut result = BatchResult::new(batch_id, batch_size, 0);
        
        // 并行处理批次中的项目
        let mut handles = Vec::new();
        
        for item in items {
            let storage_engine = self.storage_engine.clone();
            let handle = tokio::spawn(async move {
                Self::process_item(storage_engine, item).await
            });
            handles.push(handle);
        }
        
        // 等待所有项目处理完成
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => result.record_success(),
                Ok(Err(e)) => result.record_failure(e.to_string()),
                Err(e) => result.record_failure(format!("Task join error: {}", e)),
            }
        }
        
        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.record_batch(&result);
        }
        
        Ok(result)
    }
    
    /// 处理单个项目
    async fn process_item(storage_engine: Arc<dyn SimpleStorage>, item: BatchItem) -> Result<()> {
        // 只处理有效的项目
        if !item.is_valid() {
            return Err(Error::validation("Item validation failed"));
        }
        
        // 将数据序列化为字节
        let key = item.item_id.as_bytes();
        let value = bincode::serialize(&item.parsed_data.value)
            .map_err(|e| Error::serialization(e.to_string()))?;
        
        // 存储到存储引擎
        storage_engine.put(key, &value).await?;
        
        Ok(())
    }
    
    /// 启动批量处理器
    pub fn start(self: Arc<Self>, mut receiver: mpsc::Receiver<BatchItem>) -> Result<()> {
        let processor = self.clone();

        tokio::spawn(async move {
            while let Some(item) = receiver.recv().await {
                if let Err(e) = processor.add_item(item).await {
                    tracing::error!("Failed to process batch item: {}", e);
                }
            }

            // 处理剩余的批次
            if let Err(e) = processor.flush().await {
                tracing::error!("Failed to flush final batch: {}", e);
            }
        });

        Ok(())
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> BatchProcessorStats {
        self.stats.read().await.clone()
    }
    
    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = BatchProcessorStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_result() {
        let mut result = BatchResult::new("test".to_string(), 10, 1000);
        
        result.record_success();
        result.record_success();
        result.record_failure("error".to_string());
        
        assert_eq!(result.processed_count, 3);
        assert_eq!(result.success_count, 2);
        assert_eq!(result.failure_count, 1);
        assert_eq!(result.success_rate(), 2.0 / 3.0);
        assert_eq!(result.throughput(), 3.0);
    }

    #[tokio::test]
    async fn test_batch_processor_stats() {
        let mut stats = BatchProcessorStats::default();
        
        let result1 = BatchResult::new("batch1".to_string(), 5, 1000);
        let mut result1 = result1;
        result1.record_success();
        result1.record_success();
        
        let result2 = BatchResult::new("batch2".to_string(), 3, 500);
        let mut result2 = result2;
        result2.record_success();
        result2.record_failure("error".to_string());
        
        stats.record_batch(&result1);
        stats.record_batch(&result2);
        
        assert_eq!(stats.batches_processed, 2);
        assert_eq!(stats.total_messages, 4);
        assert_eq!(stats.successful_messages, 3);
        assert_eq!(stats.failed_messages, 1);
        assert_eq!(stats.success_rate(), 0.75);
    }
}
