//! Storage tier management

use crate::engine::{StorageEngine, StorageEngineType, StorageStats};
use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// 存储层级
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageTier {
    /// L1: 超热缓存 (内存) - 最近访问的数据
    L1,
    /// L2: 热数据缓存 (redb) - 频繁访问的数据
    L2,
    /// L3: 温数据存储 (DuckDB) - 定期访问的数据
    L3,
    /// L4: 冷数据存储 (RocksDB) - 归档数据
    L4,
}

impl StorageTier {
    /// 获取层级优先级（数字越小优先级越高）
    pub fn priority(&self) -> u8 {
        match self {
            StorageTier::L1 => 1,
            StorageTier::L2 => 2,
            StorageTier::L3 => 3,
            StorageTier::L4 => 4,
        }
    }
    
    /// 获取默认引擎类型
    pub fn default_engine_type(&self) -> StorageEngineType {
        match self {
            StorageTier::L1 => StorageEngineType::Memory,
            StorageTier::L2 => StorageEngineType::Redb,
            StorageTier::L3 => StorageEngineType::DuckDB,
            StorageTier::L4 => StorageEngineType::RocksDB,
        }
    }
    
    /// 获取层级名称
    pub fn name(&self) -> &'static str {
        match self {
            StorageTier::L1 => "L1-UltraHot",
            StorageTier::L2 => "L2-Hot",
            StorageTier::L3 => "L3-Warm",
            StorageTier::L4 => "L4-Cold",
        }
    }
}

/// 层级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    /// 层级
    pub tier: StorageTier,
    /// 引擎类型
    pub engine_type: StorageEngineType,
    /// 最大大小（字节）
    pub max_size: Option<usize>,
    /// 数据保留时间
    pub retention_duration: Option<chrono::Duration>,
    /// 自动迁移阈值
    pub migration_threshold: f64,
    /// 引擎配置
    pub engine_config: HashMap<String, String>,
    /// 是否启用
    pub enabled: bool,
}

impl TierConfig {
    /// 创建新的层级配置
    pub fn new(tier: StorageTier) -> Self {
        Self {
            engine_type: tier.default_engine_type(),
            tier,
            max_size: None,
            retention_duration: None,
            migration_threshold: 0.8, // 80%使用率时触发迁移
            engine_config: HashMap::new(),
            enabled: true,
        }
    }
    
    /// 设置最大大小
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size);
        self
    }
    
    /// 设置保留时间
    pub fn with_retention(mut self, duration: chrono::Duration) -> Self {
        self.retention_duration = Some(duration);
        self
    }
    
    /// 设置迁移阈值
    pub fn with_migration_threshold(mut self, threshold: f64) -> Self {
        self.migration_threshold = threshold;
        self
    }
    
    /// 添加引擎配置
    pub fn with_engine_config(mut self, key: String, value: String) -> Self {
        self.engine_config.insert(key, value);
        self
    }
}

/// 数据访问模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    /// 最后访问时间
    pub last_access: DateTime<Utc>,
    /// 访问次数
    pub access_count: u64,
    /// 访问频率（次/小时）
    pub access_frequency: f64,
    /// 数据大小
    pub data_size: usize,
    /// 数据热度评分
    pub heat_score: f64,
}

impl AccessPattern {
    /// 创建新的访问模式
    pub fn new(data_size: usize) -> Self {
        Self {
            last_access: Utc::now(),
            access_count: 1,
            access_frequency: 0.0,
            data_size,
            heat_score: 1.0,
        }
    }
    
    /// 记录访问
    pub fn record_access(&mut self) {
        let now = Utc::now();
        let time_diff = now.signed_duration_since(self.last_access);
        
        self.access_count += 1;
        self.last_access = now;
        
        // 计算访问频率（次/小时）
        if time_diff.num_hours() > 0 {
            self.access_frequency = self.access_count as f64 / time_diff.num_hours() as f64;
        }
        
        // 更新热度评分
        self.update_heat_score();
    }
    
    /// 更新热度评分
    fn update_heat_score(&mut self) {
        let now = Utc::now();
        let hours_since_access = now.signed_duration_since(self.last_access).num_hours() as f64;
        
        // 热度评分基于访问频率和时间衰减
        let time_decay = (-hours_since_access / 24.0).exp(); // 24小时衰减
        self.heat_score = self.access_frequency * time_decay;
    }
    
    /// 判断应该在哪个层级
    pub fn recommended_tier(&self) -> StorageTier {
        if self.heat_score > 10.0 {
            StorageTier::L1
        } else if self.heat_score > 1.0 {
            StorageTier::L2
        } else if self.heat_score > 0.1 {
            StorageTier::L3
        } else {
            StorageTier::L4
        }
    }
}

/// 层级管理器
pub struct TierManager {
    /// 层级配置
    tiers: HashMap<StorageTier, TierConfig>,
    /// 存储引擎
    engines: HashMap<StorageTier, Arc<RwLock<Box<dyn StorageEngine>>>>,
    /// 访问模式跟踪
    access_patterns: Arc<RwLock<HashMap<Vec<u8>, AccessPattern>>>,
    /// 迁移任务队列
    migration_queue: Arc<RwLock<Vec<MigrationTask>>>,
}

/// 迁移任务
#[derive(Debug, Clone)]
pub struct MigrationTask {
    /// 键
    pub key: Vec<u8>,
    /// 源层级
    pub from_tier: StorageTier,
    /// 目标层级
    pub to_tier: StorageTier,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 优先级
    pub priority: u8,
}

impl TierManager {
    /// 创建新的层级管理器
    pub fn new() -> Self {
        Self {
            tiers: HashMap::new(),
            engines: HashMap::new(),
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            migration_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 添加层级配置
    pub fn add_tier(&mut self, config: TierConfig) {
        self.tiers.insert(config.tier.clone(), config);
    }
    
    /// 初始化所有层级
    pub async fn initialize(&mut self) -> Result<()> {
        for (tier, config) in &self.tiers {
            if !config.enabled {
                continue;
            }
            
            let engine = crate::engine::StorageEngineFactory::create_engine(
                config.engine_type.clone(),
                config.engine_config.clone(),
            ).await?;
            
            self.engines.insert(tier.clone(), Arc::new(RwLock::new(engine)));
        }
        
        Ok(())
    }
    
    /// 获取数据
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // 按优先级顺序查找
        let mut tiers: Vec<_> = self.tiers.keys().collect();
        tiers.sort_by_key(|t| t.priority());
        
        for tier in tiers {
            if let Some(engine) = self.engines.get(tier) {
                let engine_guard = engine.read().await;
                if let Ok(Some(value)) = engine_guard.get(key).await {
                    // 记录访问模式
                    self.record_access(key, value.len()).await;
                    
                    // 如果数据在较低层级找到，考虑提升到更高层级
                    if tier.priority() > 1 {
                        self.schedule_promotion(key.to_vec(), tier.clone()).await;
                    }
                    
                    return Ok(Some(value));
                }
            }
        }
        
        Ok(None)
    }
    
    /// 设置数据
    pub async fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        // 根据数据大小和访问模式决定初始层级
        let target_tier = self.determine_initial_tier(key, value.len()).await;
        
        if let Some(engine) = self.engines.get(&target_tier) {
            let engine_guard = engine.read().await;
            engine_guard.put(key, value).await?;
            
            // 记录访问模式
            self.record_access(key, value.len()).await;
        }
        
        Ok(())
    }
    
    /// 删除数据
    pub async fn delete(&self, key: &[u8]) -> Result<()> {
        // 从所有层级删除
        for engine in self.engines.values() {
            let engine_guard = engine.read().await;
            let _ = engine_guard.delete(key).await; // 忽略错误，因为数据可能不在所有层级
        }
        
        // 清除访问模式
        self.access_patterns.write().await.remove(key);
        
        Ok(())
    }
    
    /// 记录访问模式
    async fn record_access(&self, key: &[u8], data_size: usize) {
        let mut patterns = self.access_patterns.write().await;
        let pattern = patterns.entry(key.to_vec()).or_insert_with(|| AccessPattern::new(data_size));
        pattern.record_access();
    }
    
    /// 确定初始层级
    async fn determine_initial_tier(&self, key: &[u8], data_size: usize) -> StorageTier {
        // 检查是否有历史访问模式
        if let Some(pattern) = self.access_patterns.read().await.get(key) {
            return pattern.recommended_tier();
        }
        
        // 新数据默认放在L2
        StorageTier::L2
    }
    
    /// 调度提升任务
    async fn schedule_promotion(&self, key: Vec<u8>, current_tier: StorageTier) {
        let target_tier = match current_tier {
            StorageTier::L4 => StorageTier::L3,
            StorageTier::L3 => StorageTier::L2,
            StorageTier::L2 => StorageTier::L1,
            StorageTier::L1 => return, // 已经在最高层级
        };
        
        let task = MigrationTask {
            key,
            from_tier: current_tier,
            to_tier: target_tier,
            created_at: Utc::now(),
            priority: 1,
        };
        
        self.migration_queue.write().await.push(task);
    }
    
    /// 执行迁移任务
    pub async fn process_migrations(&self) -> Result<()> {
        let mut queue = self.migration_queue.write().await;
        
        while let Some(task) = queue.pop() {
            self.migrate_data(task).await?;
        }
        
        Ok(())
    }
    
    /// 迁移数据
    async fn migrate_data(&self, task: MigrationTask) -> Result<()> {
        // 从源层级读取数据
        let value = if let Some(source_engine) = self.engines.get(&task.from_tier) {
            let engine_guard = source_engine.read().await;
            engine_guard.get(&task.key).await?
        } else {
            return Ok(()); // 源引擎不存在
        };
        
        if let Some(value) = value {
            // 写入目标层级
            if let Some(target_engine) = self.engines.get(&task.to_tier) {
                let engine_guard = target_engine.read().await;
                engine_guard.put(&task.key, &value).await?;
            }
            
            // 从源层级删除（可选，取决于策略）
            if task.to_tier.priority() < task.from_tier.priority() {
                if let Some(source_engine) = self.engines.get(&task.from_tier) {
                    let engine_guard = source_engine.read().await;
                    let _ = engine_guard.delete(&task.key).await;
                }
            }
        }
        
        Ok(())
    }
    
    /// 获取层级统计
    pub async fn get_tier_stats(&self) -> Result<HashMap<StorageTier, StorageStats>> {
        let mut stats = HashMap::new();
        
        for (tier, engine) in &self.engines {
            let engine_guard = engine.read().await;
            let tier_stats = engine_guard.stats().await?;
            stats.insert(tier.clone(), tier_stats);
        }
        
        Ok(stats)
    }
    
    /// 获取访问模式统计
    pub async fn get_access_patterns_count(&self) -> usize {
        self.access_patterns.read().await.len()
    }
    
    /// 获取迁移队列长度
    pub async fn get_migration_queue_length(&self) -> usize {
        self.migration_queue.read().await.len()
    }
}

impl Default for TierManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_tier_priority() {
        assert_eq!(StorageTier::L1.priority(), 1);
        assert_eq!(StorageTier::L2.priority(), 2);
        assert_eq!(StorageTier::L3.priority(), 3);
        assert_eq!(StorageTier::L4.priority(), 4);
    }

    #[test]
    fn test_tier_config() {
        let config = TierConfig::new(StorageTier::L1)
            .with_max_size(1024 * 1024 * 1024)
            .with_migration_threshold(0.9);
        
        assert_eq!(config.tier, StorageTier::L1);
        assert_eq!(config.max_size, Some(1024 * 1024 * 1024));
        assert_eq!(config.migration_threshold, 0.9);
    }

    #[test]
    fn test_access_pattern() {
        let mut pattern = AccessPattern::new(1024);
        assert_eq!(pattern.access_count, 1);
        assert_eq!(pattern.data_size, 1024);
        
        pattern.record_access();
        assert_eq!(pattern.access_count, 2);
        
        let tier = pattern.recommended_tier();
        assert!(matches!(tier, StorageTier::L1 | StorageTier::L2 | StorageTier::L3 | StorageTier::L4));
    }

    #[test]
    fn test_tier_manager_creation() {
        let manager = TierManager::new();
        assert_eq!(manager.tiers.len(), 0);
        assert_eq!(manager.engines.len(), 0);
    }
}
