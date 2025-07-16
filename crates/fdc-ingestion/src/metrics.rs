//! Ingestion metrics collection and reporting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::sync::Arc; // 暂时未使用
// use tokio::sync::RwLock; // 暂时未使用

/// 接入指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IngestionMetrics {
    /// 接收的消息数
    pub messages_received: u64,
    /// 解析的消息数
    pub messages_parsed: u64,
    /// 验证的消息数
    pub messages_validated: u64,
    /// 存储的消息数
    pub messages_stored: u64,
    /// 丢弃的消息数
    pub messages_dropped: u64,
    /// 错误消息数
    pub messages_errored: u64,
    /// 总处理时间（微秒）
    pub total_processing_time_us: u64,
    /// 平均处理时间（微秒）
    pub avg_processing_time_us: f64,
    /// 吞吐量（消息/秒）
    pub throughput_msg_per_sec: f64,
    /// 错误率
    pub error_rate: f64,
    /// 按协议统计
    pub protocol_stats: HashMap<String, u64>,
    /// 按数据类型统计
    pub data_type_stats: HashMap<String, u64>,
}

impl IngestionMetrics {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn record_message_received(&mut self, protocol: &str) {
        self.messages_received += 1;
        *self.protocol_stats.entry(protocol.to_string()).or_insert(0) += 1;
    }
    
    pub fn record_message_parsed(&mut self, data_type: &str) {
        self.messages_parsed += 1;
        *self.data_type_stats.entry(data_type.to_string()).or_insert(0) += 1;
    }
    
    pub fn record_message_validated(&mut self) {
        self.messages_validated += 1;
    }
    
    pub fn record_message_stored(&mut self) {
        self.messages_stored += 1;
    }
    
    pub fn record_message_dropped(&mut self) {
        self.messages_dropped += 1;
    }
    
    pub fn record_message_error(&mut self) {
        self.messages_errored += 1;
        self.update_error_rate();
    }
    
    pub fn record_processing_time(&mut self, time_us: u64) {
        self.total_processing_time_us += time_us;
        let total_messages = self.messages_received;
        if total_messages > 0 {
            self.avg_processing_time_us = self.total_processing_time_us as f64 / total_messages as f64;
        }
    }
    
    fn update_error_rate(&mut self) {
        if self.messages_received > 0 {
            self.error_rate = self.messages_errored as f64 / self.messages_received as f64;
        }
    }
    
    pub fn calculate_throughput(&mut self, time_window_secs: f64) {
        if time_window_secs > 0.0 {
            self.throughput_msg_per_sec = self.messages_received as f64 / time_window_secs;
        }
    }
}
