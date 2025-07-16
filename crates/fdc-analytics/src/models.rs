//! Analytics data models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResult<T> {
    /// 结果数据
    pub data: T,
    /// 计算时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 计算耗时（微秒）
    pub computation_time_us: u64,
    /// 置信度
    pub confidence: Option<f64>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

impl<T> AnalyticsResult<T> {
    /// 创建新的分析结果
    pub fn new(data: T, computation_time_us: u64) -> Self {
        Self {
            data,
            timestamp: chrono::Utc::now(),
            computation_time_us,
            confidence: None,
            metadata: HashMap::new(),
        }
    }
    
    /// 设置置信度
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 时间序列数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 数值
    pub value: f64,
    /// 标签
    pub labels: HashMap<String, String>,
}

impl TimeSeriesData {
    /// 创建新的时间序列数据点
    pub fn new(timestamp: chrono::DateTime<chrono::Utc>, value: f64) -> Self {
        Self {
            timestamp,
            value,
            labels: HashMap::new(),
        }
    }
    
    /// 添加标签
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }
}

/// 市场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    /// 交易品种
    pub symbol: String,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量
    pub volume: f64,
    /// 成交额
    pub turnover: Option<f64>,
}

impl MarketData {
    /// 创建新的市场数据
    pub fn new(
        symbol: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        Self {
            symbol,
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            turnover: None,
        }
    }
    
    /// 设置成交额
    pub fn with_turnover(mut self, turnover: f64) -> Self {
        self.turnover = Some(turnover);
        self
    }
    
    /// 计算典型价格
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }
    
    /// 计算价格变化率
    pub fn price_change_rate(&self) -> f64 {
        if self.open == 0.0 {
            0.0
        } else {
            (self.close - self.open) / self.open
        }
    }
}

/// 技术指标结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorResult {
    /// 指标名称
    pub name: String,
    /// 指标值
    pub value: f64,
    /// 信号类型
    pub signal: SignalType,
    /// 参数
    pub parameters: HashMap<String, f64>,
}

/// 信号类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    /// 买入信号
    Buy,
    /// 卖出信号
    Sell,
    /// 持有信号
    Hold,
    /// 中性信号
    Neutral,
}

/// 风险指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    /// 风险价值（VaR）
    pub var: f64,
    /// 条件风险价值（CVaR）
    pub cvar: f64,
    /// 最大回撤
    pub max_drawdown: f64,
    /// 夏普比率
    pub sharpe_ratio: f64,
    /// 波动率
    pub volatility: f64,
    /// Beta系数
    pub beta: Option<f64>,
}

/// 聚合结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    /// 分组键
    pub group_key: HashMap<String, String>,
    /// 聚合值
    pub aggregated_values: HashMap<String, f64>,
    /// 数据点数量
    pub count: u64,
    /// 时间窗口
    pub window_start: chrono::DateTime<chrono::Utc>,
    /// 窗口结束
    pub window_end: chrono::DateTime<chrono::Utc>,
}

/// 机器学习预测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// 预测值
    pub predicted_value: f64,
    /// 预测区间
    pub prediction_interval: Option<(f64, f64)>,
    /// 模型置信度
    pub confidence_score: f64,
    /// 特征重要性
    pub feature_importance: HashMap<String, f64>,
}

/// 流处理事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    /// 事件ID
    pub event_id: String,
    /// 事件类型
    pub event_type: String,
    /// 事件数据
    pub data: serde_json::Value,
    /// 事件时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 处理时间戳
    pub processing_time: chrono::DateTime<chrono::Utc>,
}

impl StreamEvent {
    /// 创建新的流事件
    pub fn new(event_type: String, data: serde_json::Value) -> Self {
        let now = chrono::Utc::now();
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            data,
            timestamp: now,
            processing_time: now,
        }
    }
}

/// 批处理作业
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    /// 作业ID
    pub job_id: String,
    /// 作业名称
    pub job_name: String,
    /// 作业状态
    pub status: BatchJobStatus,
    /// 输入数据路径
    pub input_path: String,
    /// 输出数据路径
    pub output_path: String,
    /// 作业参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 开始时间
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 完成时间
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 批处理作业状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchJobStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_result() {
        let result = AnalyticsResult::new("test data", 1000)
            .with_confidence(0.95)
            .with_metadata("source".to_string(), serde_json::Value::String("test".to_string()));
        
        assert_eq!(result.data, "test data");
        assert_eq!(result.computation_time_us, 1000);
        assert_eq!(result.confidence, Some(0.95));
        assert!(result.metadata.contains_key("source"));
    }

    #[test]
    fn test_market_data() {
        let data = MarketData::new(
            "AAPL".to_string(),
            chrono::Utc::now(),
            100.0,
            105.0,
            98.0,
            103.0,
            1000.0,
        ).with_turnover(103000.0);
        
        assert_eq!(data.symbol, "AAPL");
        assert_eq!(data.typical_price(), (105.0 + 98.0 + 103.0) / 3.0);
        assert_eq!(data.price_change_rate(), 0.03);
        assert_eq!(data.turnover, Some(103000.0));
    }

    #[test]
    fn test_stream_event() {
        let event = StreamEvent::new(
            "market_data".to_string(),
            serde_json::json!({"symbol": "AAPL", "price": 150.0}),
        );
        
        assert_eq!(event.event_type, "market_data");
        assert!(!event.event_id.is_empty());
    }
}
