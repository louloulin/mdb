//! Analytics pipeline management

use crate::config::AnalyticsConfig;
use fdc_core::error::Result;
use serde::{Deserialize, Serialize};

/// 分析管道阶段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineStage {
    /// 数据接入
    Ingestion,
    /// 数据清洗
    Cleaning,
    /// 特征工程
    FeatureEngineering,
    /// 模型训练
    Training,
    /// 预测
    Prediction,
    /// 结果输出
    Output,
}

/// 管道状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineStatus {
    /// 待运行
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已暂停
    Paused,
}

/// 分析管道
pub struct AnalyticsPipeline {
    /// 管道ID
    pub id: String,
    /// 管道名称
    pub name: String,
    /// 配置
    config: AnalyticsConfig,
    /// 当前阶段
    current_stage: Option<PipelineStage>,
    /// 状态
    status: PipelineStatus,
}

impl AnalyticsPipeline {
    /// 创建新的分析管道
    pub fn new(id: String, name: String, config: AnalyticsConfig) -> Self {
        Self {
            id,
            name,
            config,
            current_stage: None,
            status: PipelineStatus::Pending,
        }
    }
    
    /// 启动管道
    pub async fn start(&mut self) -> Result<()> {
        self.status = PipelineStatus::Running;
        self.current_stage = Some(PipelineStage::Ingestion);
        tracing::info!("Analytics pipeline '{}' started", self.name);
        Ok(())
    }
    
    /// 停止管道
    pub async fn stop(&mut self) -> Result<()> {
        self.status = PipelineStatus::Paused;
        tracing::info!("Analytics pipeline '{}' stopped", self.name);
        Ok(())
    }
    
    /// 获取状态
    pub fn get_status(&self) -> &PipelineStatus {
        &self.status
    }
    
    /// 获取当前阶段
    pub fn get_current_stage(&self) -> &Option<PipelineStage> {
        &self.current_stage
    }
    
    /// 推进到下一阶段
    pub fn advance_stage(&mut self) -> Result<()> {
        self.current_stage = match &self.current_stage {
            Some(PipelineStage::Ingestion) => Some(PipelineStage::Cleaning),
            Some(PipelineStage::Cleaning) => Some(PipelineStage::FeatureEngineering),
            Some(PipelineStage::FeatureEngineering) => Some(PipelineStage::Training),
            Some(PipelineStage::Training) => Some(PipelineStage::Prediction),
            Some(PipelineStage::Prediction) => Some(PipelineStage::Output),
            Some(PipelineStage::Output) => {
                self.status = PipelineStatus::Completed;
                None
            }
            None => Some(PipelineStage::Ingestion),
        };
        Ok(())
    }
}

/// 管道管理器
pub struct PipelineManager {
    pipelines: Vec<AnalyticsPipeline>,
}

impl PipelineManager {
    /// 创建新的管道管理器
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }
    
    /// 添加管道
    pub fn add_pipeline(&mut self, pipeline: AnalyticsPipeline) {
        self.pipelines.push(pipeline);
    }
    
    /// 获取管道
    pub fn get_pipeline(&mut self, id: &str) -> Option<&mut AnalyticsPipeline> {
        self.pipelines.iter_mut().find(|p| p.id == id)
    }
    
    /// 获取所有管道
    pub fn get_all_pipelines(&self) -> &[AnalyticsPipeline] {
        &self.pipelines
    }
    
    /// 启动所有管道
    pub async fn start_all(&mut self) -> Result<()> {
        for pipeline in &mut self.pipelines {
            pipeline.start().await?;
        }
        Ok(())
    }
    
    /// 停止所有管道
    pub async fn stop_all(&mut self) -> Result<()> {
        for pipeline in &mut self.pipelines {
            pipeline.stop().await?;
        }
        Ok(())
    }
}

impl Default for PipelineManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_lifecycle() {
        let config = AnalyticsConfig::default();
        let mut pipeline = AnalyticsPipeline::new(
            "test-pipeline".to_string(),
            "Test Pipeline".to_string(),
            config,
        );
        
        assert!(matches!(pipeline.get_status(), PipelineStatus::Pending));
        assert!(pipeline.get_current_stage().is_none());
        
        pipeline.start().await.unwrap();
        assert!(matches!(pipeline.get_status(), PipelineStatus::Running));
        assert!(matches!(pipeline.get_current_stage(), Some(PipelineStage::Ingestion)));
        
        pipeline.advance_stage().unwrap();
        assert!(matches!(pipeline.get_current_stage(), Some(PipelineStage::Cleaning)));
        
        pipeline.stop().await.unwrap();
        assert!(matches!(pipeline.get_status(), PipelineStatus::Paused));
    }

    #[tokio::test]
    async fn test_pipeline_manager() {
        let mut manager = PipelineManager::new();
        
        let config = AnalyticsConfig::default();
        let pipeline = AnalyticsPipeline::new(
            "test-pipeline".to_string(),
            "Test Pipeline".to_string(),
            config,
        );
        
        manager.add_pipeline(pipeline);
        assert_eq!(manager.get_all_pipelines().len(), 1);
        
        let pipeline = manager.get_pipeline("test-pipeline").unwrap();
        assert_eq!(pipeline.id, "test-pipeline");
        
        manager.start_all().await.unwrap();
        let pipeline = manager.get_pipeline("test-pipeline").unwrap();
        assert!(matches!(pipeline.get_status(), PipelineStatus::Running));
    }
}
