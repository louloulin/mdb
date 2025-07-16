//! Data parsing and format conversion

use crate::{config::ParserConfig, receiver::ReceivedData};
use fdc_core::{error::{Error, Result}, types::Value};
use fdc_types::TypeRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 解析后的数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedData {
    /// 解析后的值
    pub value: Value,
    /// 数据类型
    pub data_type: String,
    /// 解析时间戳
    pub parsed_at: chrono::DateTime<chrono::Utc>,
    /// 原始数据大小
    pub original_size: usize,
    /// 解析后大小
    pub parsed_size: usize,
    /// 解析耗时（微秒）
    pub parse_time_us: u64,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl ParsedData {
    /// 创建新的解析数据
    pub fn new(value: Value, data_type: String, original_size: usize, parse_time_us: u64) -> Self {
        let parsed_size = Self::estimate_size(&value);
        Self {
            value,
            data_type,
            parsed_at: chrono::Utc::now(),
            original_size,
            parsed_size,
            parse_time_us,
            metadata: HashMap::new(),
        }
    }
    
    /// 估算解析后数据大小
    fn estimate_size(value: &Value) -> usize {
        match value {
            Value::Null => 0,
            Value::Bool(_) => 1,
            Value::Int8(_) => 1,
            Value::Int16(_) => 2,
            Value::Int32(_) => 4,
            Value::Int64(_) => 8,
            Value::Int128(_) => 16,
            Value::UInt8(_) => 1,
            Value::UInt16(_) => 2,
            Value::UInt32(_) => 4,
            Value::UInt64(_) => 8,
            Value::UInt128(_) => 16,
            Value::Float32(_) => 4,
            Value::Float64(_) => 8,
            Value::String(s) => s.len(),
            Value::Binary(b) => b.len(),
            Value::Array(arr) => arr.iter().map(Self::estimate_size).sum(),
            Value::Struct(map) | Value::Map(map) => {
                map.iter().map(|(k, v)| k.len() + Self::estimate_size(v)).sum()
            }
            _ => 0,
        }
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// 获取压缩比
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            self.parsed_size as f64 / self.original_size as f64
        }
    }
}

/// 解析器统计信息
#[derive(Debug, Clone, Default)]
pub struct ParserStats {
    /// 解析的消息数
    pub messages_parsed: u64,
    /// 解析成功数
    pub parse_successes: u64,
    /// 解析失败数
    pub parse_failures: u64,
    /// 总解析时间（微秒）
    pub total_parse_time_us: u64,
    /// 平均解析时间（微秒）
    pub avg_parse_time_us: f64,
    /// 最小解析时间（微秒）
    pub min_parse_time_us: u64,
    /// 最大解析时间（微秒）
    pub max_parse_time_us: u64,
    /// 处理的字节数
    pub bytes_processed: u64,
    /// 解析吞吐量（消息/秒）
    pub throughput_msg_per_sec: f64,
}

impl ParserStats {
    /// 记录解析结果
    pub fn record_parse(&mut self, success: bool, parse_time_us: u64, bytes: usize) {
        self.messages_parsed += 1;
        self.bytes_processed += bytes as u64;
        self.total_parse_time_us += parse_time_us;
        
        if success {
            self.parse_successes += 1;
        } else {
            self.parse_failures += 1;
        }
        
        // 更新时间统计
        if self.messages_parsed == 1 {
            self.min_parse_time_us = parse_time_us;
            self.max_parse_time_us = parse_time_us;
        } else {
            self.min_parse_time_us = self.min_parse_time_us.min(parse_time_us);
            self.max_parse_time_us = self.max_parse_time_us.max(parse_time_us);
        }
        
        self.avg_parse_time_us = self.total_parse_time_us as f64 / self.messages_parsed as f64;
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.messages_parsed == 0 {
            0.0
        } else {
            self.parse_successes as f64 / self.messages_parsed as f64
        }
    }
}

/// 数据解析器
pub struct DataParser {
    /// 配置
    config: ParserConfig,
    /// 类型注册表
    type_registry: Arc<TypeRegistry>,
    /// 解析器统计
    stats: Arc<RwLock<ParserStats>>,
}

impl DataParser {
    /// 创建新的数据解析器
    pub fn new(config: ParserConfig, type_registry: Arc<TypeRegistry>) -> Self {
        Self {
            config,
            type_registry,
            stats: Arc::new(RwLock::new(ParserStats::default())),
        }
    }
    
    /// 解析接收到的数据
    pub async fn parse(&self, received_data: ReceivedData) -> Result<ParsedData> {
        let start_time = std::time::Instant::now();
        
        let result = match self.config.parser_type.as_str() {
            "json" => self.parse_json(&received_data.data).await,
            "binary" => self.parse_binary(&received_data.data).await,
            "csv" => self.parse_csv(&received_data.data).await,
            "fix" => self.parse_fix(&received_data.data).await,
            _ => Err(Error::unimplemented(format!("Parser type: {}", self.config.parser_type))),
        };
        
        let parse_time_us = start_time.elapsed().as_micros() as u64;
        let success = result.is_ok();
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.record_parse(success, parse_time_us, received_data.size);
        }
        
        match result {
            Ok(value) => {
                let parsed_data = ParsedData::new(
                    value,
                    self.config.parser_type.clone(),
                    received_data.size,
                    parse_time_us,
                );
                Ok(parsed_data)
            }
            Err(e) => Err(e),
        }
    }
    
    /// 解析JSON数据
    async fn parse_json(&self, data: &[u8]) -> Result<Value> {
        let json_str = String::from_utf8(data.to_vec())
            .map_err(|e| Error::validation(format!("Invalid UTF-8 in JSON data: {}", e)))?;
        
        let json_value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| Error::validation(format!("Invalid JSON: {}", e)))?;
        
        self.json_to_value(json_value)
    }
    
    /// 将JSON值转换为内部Value类型
    fn json_to_value(&self, json_value: serde_json::Value) -> Result<Value> {
        match json_value {
            serde_json::Value::Null => Ok(Value::Null),
            serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Int64(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Float64(f))
                } else {
                    Err(Error::validation("Invalid number format".to_string()))
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(s)),
            serde_json::Value::Array(arr) => {
                let values: Result<Vec<Value>> = arr.into_iter()
                    .map(|v| self.json_to_value(v))
                    .collect();
                Ok(Value::Array(values?))
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (key, value) in obj {
                    map.insert(key, self.json_to_value(value)?);
                }
                Ok(Value::Struct(map))
            }
        }
    }
    
    /// 解析二进制数据
    async fn parse_binary(&self, data: &[u8]) -> Result<Value> {
        // 尝试使用bincode反序列化
        bincode::deserialize::<Value>(data)
            .map_err(|e| Error::validation(format!("Binary deserialization failed: {}", e)))
    }
    
    /// 解析CSV数据
    async fn parse_csv(&self, data: &[u8]) -> Result<Value> {
        let csv_str = String::from_utf8(data.to_vec())
            .map_err(|e| Error::validation(format!("Invalid UTF-8 in CSV data: {}", e)))?;
        
        let mut reader = csv::Reader::from_reader(csv_str.as_bytes());
        let mut records = Vec::new();
        
        for result in reader.records() {
            let record = result.map_err(|e| Error::validation(format!("CSV parsing error: {}", e)))?;
            let values: Vec<Value> = record.iter()
                .map(|field| Value::String(field.to_string()))
                .collect();
            records.push(Value::Array(values));
        }
        
        Ok(Value::Array(records))
    }
    
    /// 解析FIX协议数据
    async fn parse_fix(&self, data: &[u8]) -> Result<Value> {
        let fix_str = String::from_utf8(data.to_vec())
            .map_err(|e| Error::validation(format!("Invalid UTF-8 in FIX data: {}", e)))?;
        
        let mut fields = HashMap::new();
        
        // 简单的FIX解析（按SOH分隔符分割）
        for field in fix_str.split('\x01') {
            if let Some(eq_pos) = field.find('=') {
                let tag = &field[..eq_pos];
                let value = &field[eq_pos + 1..];
                fields.insert(tag.to_string(), Value::String(value.to_string()));
            }
        }
        
        Ok(Value::Struct(fields))
    }
    
    /// 获取解析器统计信息
    pub async fn get_stats(&self) -> ParserStats {
        self.stats.read().await.clone()
    }
    
    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ParserStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fdc_types::TypeRegistryConfig;

    fn create_test_parser() -> DataParser {
        let config = ParserConfig::default();
        let type_registry = Arc::new(TypeRegistry::new(TypeRegistryConfig::default()));
        DataParser::new(config, type_registry)
    }

    #[tokio::test]
    async fn test_json_parsing() {
        let parser = create_test_parser();
        let json_data = r#"{"name": "test", "value": 42, "active": true}"#;
        
        let result = parser.parse_json(json_data.as_bytes()).await;
        assert!(result.is_ok());
        
        if let Ok(Value::Struct(map)) = result {
            assert_eq!(map.len(), 3);
            assert!(map.contains_key("name"));
            assert!(map.contains_key("value"));
            assert!(map.contains_key("active"));
        }
    }

    #[tokio::test]
    async fn test_csv_parsing() {
        let parser = create_test_parser();
        let csv_data = "name,age,city\nJohn,30,NYC\nJane,25,LA";
        
        let result = parser.parse_csv(csv_data.as_bytes()).await;
        assert!(result.is_ok());
        
        if let Ok(Value::Array(records)) = result {
            assert_eq!(records.len(), 2); // 2 data rows (header is not included in this simple implementation)
        }
    }

    #[test]
    fn test_parsed_data_creation() {
        let value = Value::String("test".to_string());
        let data_type = "json".to_string();
        let original_size = 100;
        let parse_time = 1000;
        
        let parsed = ParsedData::new(value, data_type.clone(), original_size, parse_time);
        
        assert_eq!(parsed.data_type, data_type);
        assert_eq!(parsed.original_size, original_size);
        assert_eq!(parsed.parse_time_us, parse_time);
        assert!(parsed.parsed_size > 0);
    }

    #[test]
    fn test_parser_stats() {
        let mut stats = ParserStats::default();
        
        stats.record_parse(true, 1000, 100);
        stats.record_parse(false, 2000, 200);
        stats.record_parse(true, 1500, 150);
        
        assert_eq!(stats.messages_parsed, 3);
        assert_eq!(stats.parse_successes, 2);
        assert_eq!(stats.parse_failures, 1);
        assert_eq!(stats.success_rate(), 2.0 / 3.0);
        assert_eq!(stats.avg_parse_time_us, 1500.0);
        assert_eq!(stats.min_parse_time_us, 1000);
        assert_eq!(stats.max_parse_time_us, 2000);
    }
}
