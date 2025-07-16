//! Type conversion system

use fdc_core::{error::{Error, Result}, types::Value};
use serde::{Deserialize, Serialize};

/// 转换错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionError {
    pub from_type: String,
    pub to_type: String,
    pub message: String,
}

/// 转换规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversionRule {
    Direct,
    Lossy,
    Custom(String),
}

/// 类型转换器
pub struct TypeConverter;

impl TypeConverter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn convert(&self, value: &Value, target_type: &str) -> Result<Value> {
        match (value, target_type) {
            (Value::Int32(v), "i64") => Ok(Value::Int64(*v as i64)),
            (Value::Int64(v), "i32") => Ok(Value::Int32(*v as i32)),
            (Value::Float32(v), "f64") => Ok(Value::Float64(*v as f64)),
            (Value::Float64(v), "f32") => Ok(Value::Float32(*v as f32)),
            _ => Err(Error::type_error(format!("Cannot convert {:?} to {}", value, target_type))),
        }
    }
}

impl Default for TypeConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let converter = TypeConverter::new();
        let value = Value::Int32(42);
        let result = converter.convert(&value, "i64").unwrap();
        assert_eq!(result, Value::Int64(42));
    }
}
