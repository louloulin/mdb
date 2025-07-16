//! Core data types for Financial Data Center

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// 纳秒级时间戳
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TimestampNs(pub i64);

impl TimestampNs {
    /// 创建当前时间的时间戳
    pub fn now() -> Self {
        let now = Utc::now();
        Self(now.timestamp_nanos_opt().unwrap_or(0))
    }
    
    /// 从纳秒值创建时间戳
    pub fn from_nanos(nanos: i64) -> Self {
        Self(nanos)
    }
    
    /// 获取纳秒值
    pub fn as_nanos(&self) -> i64 {
        self.0
    }
    
    /// 转换为DateTime
    pub fn to_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_nanos(self.0).into()
    }
}

impl fmt::Display for TimestampNs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(dt) = self.to_datetime() {
            write!(f, "{}", dt.format("%Y-%m-%d %H:%M:%S%.9f UTC"))
        } else {
            write!(f, "Invalid timestamp: {}", self.0)
        }
    }
}

/// 自定义符号类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self(symbol.into().to_uppercase())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// 自定义价格类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price(Decimal);

impl Price {
    pub fn new(value: Decimal) -> Self {
        Self(value)
    }
    
    pub fn from_f64(value: f64) -> Option<Self> {
        Decimal::try_from(value).ok().map(Self)
    }
    
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }
    
    pub fn to_f64(&self) -> f64 {
        self.0.try_into().unwrap_or(0.0)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 自定义成交量类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Volume(u64);

impl Volume {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 交易所ID类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExchangeId(u16);

impl ExchangeId {
    pub fn new(id: u16) -> Self {
        Self(id)
    }
    
    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    Trade,
    Quote,
    OrderBook,
    Index,
    Custom(u8),
}

/// 序列号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    pub fn new(seq: u64) -> Self {
        Self(seq)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

/// 自定义字段容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFields {
    fields: HashMap<String, Value>,
    type_info: Option<TypeInfo>,
}

impl CustomFields {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            type_info: None,
        }
    }
    
    pub fn insert(&mut self, key: String, value: Value) {
        self.fields.insert(key, value);
    }
    
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }
    
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl Default for CustomFields {
    fn default() -> Self {
        Self::new()
    }
}

/// 类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub type_id: TypeId,
    pub name: String,
    pub version: String,
}

/// 类型ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeId(Uuid);

impl TypeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for TypeId {
    fn default() -> Self {
        Self::new()
    }
}

/// 支持动态类型的值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    // 基础类型
    Null,
    Bool(bool),
    Int8(i8), Int16(i16), Int32(i32), Int64(i64), Int128(i128),
    UInt8(u8), UInt16(u16), UInt32(u32), UInt64(u64), UInt128(u128),
    Float32(f32), Float64(f64),
    Decimal(Decimal),
    String(String), 
    Binary(Vec<u8>),
    
    // 时间类型
    Timestamp(TimestampNs),
    
    // 复合类型
    Array(Vec<Value>), 
    List(Vec<Value>),
    Struct(HashMap<String, Value>),
    Map(HashMap<String, Value>),
    
    // 金融专用类型
    Price(Price), 
    Volume(Volume), 
    Symbol(Symbol),
    ExchangeId(ExchangeId),
    
    // 自定义类型（通过WASM定义）
    Custom(CustomValue),
}

/// 自定义值类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomValue {
    pub type_id: TypeId,
    pub data: Vec<u8>,
    pub wasm_module: Option<String>,
}

/// 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: TimestampNs,
    pub updated_at: TimestampNs,
    pub version: u32,
    pub tags: HashMap<String, String>,
}

impl Metadata {
    pub fn new() -> Self {
        let now = TimestampNs::now();
        Self {
            created_at: now,
            updated_at: now,
            version: 1,
            tags: HashMap::new(),
        }
    }
    
    pub fn update(&mut self) {
        self.updated_at = TimestampNs::now();
        self.version += 1;
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

/// 增强的核心数据类型（支持自定义类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickData {
    pub timestamp: TimestampNs,        // 纳秒级时间戳类型
    pub symbol: Symbol,                // 自定义符号类型
    pub price: Price,                  // 自定义价格类型
    pub volume: Volume,                // 自定义成交量类型
    pub bid_price: Option<Price>,
    pub ask_price: Option<Price>,
    pub bid_size: Option<Volume>,
    pub ask_size: Option<Volume>,
    pub exchange_id: ExchangeId,       // 自定义交易所ID类型
    pub message_type: MessageType,
    pub sequence_number: SequenceNumber,
    
    // 扩展字段
    pub custom_fields: CustomFields,   // 用户自定义字段
    pub metadata: Metadata,            // 元数据
    pub wasm_processed: bool,          // 是否经过WASM处理
}

impl TickData {
    pub fn new(
        symbol: impl Into<Symbol>,
        price: Price,
        volume: Volume,
        exchange_id: ExchangeId,
        message_type: MessageType,
        sequence_number: SequenceNumber,
    ) -> Self {
        Self {
            timestamp: TimestampNs::now(),
            symbol: symbol.into(),
            price,
            volume,
            bid_price: None,
            ask_price: None,
            bid_size: None,
            ask_size: None,
            exchange_id,
            message_type,
            sequence_number,
            custom_fields: CustomFields::new(),
            metadata: Metadata::new(),
            wasm_processed: false,
        }
    }
}
