//! Batch processing engine

use crate::{config::BatchConfig, models::BatchJob};
use fdc_core::error::Result;
use serde::{Deserialize, Serialize};

/// 批处理器统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchStats {
    /// 处理的作业数
    pub jobs_processed: u64,
    /// 成功作业数
    pub jobs_success: u64,
    /// 失败作业数
    pub jobs_failed: u64,
    /// 平均处理时间（毫秒）
    pub avg_processing_time_ms: f64,
}

/// 批处理器
pub struct BatchProcessor {
    config: BatchConfig,
    stats: BatchStats,
}

impl BatchProcessor {
    /// 创建新的批处理器
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            stats: BatchStats::default(),
        }
    }
    
    /// 提交批处理作业
    pub async fn submit_job(&mut self, job: BatchJob) -> Result<String> {
        // 简化实现
        tracing::info!("Submitting batch job: {}", job.job_name);
        self.stats.jobs_processed += 1;
        self.stats.jobs_success += 1;
        Ok(job.job_id)
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> &BatchStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BatchJobStatus;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_batch_processor() {
        let config = BatchConfig::default();
        let mut processor = BatchProcessor::new(config);
        
        let job = BatchJob {
            job_id: "test-job".to_string(),
            job_name: "Test Job".to_string(),
            status: BatchJobStatus::Pending,
            input_path: "/input".to_string(),
            output_path: "/output".to_string(),
            parameters: HashMap::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };
        
        let result = processor.submit_job(job).await;
        assert!(result.is_ok());
        assert_eq!(processor.get_stats().jobs_processed, 1);
    }
}
