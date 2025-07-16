//! WASM type definitions and conversions

use fdc_core::{
    error::{Error, Result},
    types::Value,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WASM值类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WasmValue {
    /// 空值
    Null,
    /// 布尔值
    Bool(bool),
    /// 32位整数
    I32(i32),
    /// 64位整数
    I64(i64),
    /// 32位浮点数
    F32(f32),
    /// 64位浮点数
    F64(f64),
    /// 字符串
    String(String),
    /// 字节数组
    Bytes(Vec<u8>),
    /// 数组
    Array(Vec<WasmValue>),
    /// 对象
    Object(HashMap<String, WasmValue>),
}

impl WasmValue {
    /// 获取值的类型
    pub fn value_type(&self) -> WasmType {
        match self {
            WasmValue::Null => WasmType::Null,
            WasmValue::Bool(_) => WasmType::Bool,
            WasmValue::I32(_) => WasmType::I32,
            WasmValue::I64(_) => WasmType::I64,
            WasmValue::F32(_) => WasmType::F32,
            WasmValue::F64(_) => WasmType::F64,
            WasmValue::String(_) => WasmType::String,
            WasmValue::Bytes(_) => WasmType::Bytes,
            WasmValue::Array(_) => WasmType::Array,
            WasmValue::Object(_) => WasmType::Object,
        }
    }
    
    /// 检查是否为数值类型
    pub fn is_numeric(&self) -> bool {
        matches!(self, WasmValue::I32(_) | WasmValue::I64(_) | WasmValue::F32(_) | WasmValue::F64(_))
    }
    
    /// 检查是否为整数类型
    pub fn is_integer(&self) -> bool {
        matches!(self, WasmValue::I32(_) | WasmValue::I64(_))
    }
    
    /// 检查是否为浮点数类型
    pub fn is_float(&self) -> bool {
        matches!(self, WasmValue::F32(_) | WasmValue::F64(_))
    }
    
    /// 转换为i32
    pub fn as_i32(&self) -> Result<i32> {
        match self {
            WasmValue::I32(v) => Ok(*v),
            WasmValue::I64(v) => Ok(*v as i32),
            WasmValue::F32(v) => Ok(*v as i32),
            WasmValue::F64(v) => Ok(*v as i32),
            _ => Err(Error::type_error(format!("Cannot convert {:?} to i32", self.value_type()))),
        }
    }
    
    /// 转换为i64
    pub fn as_i64(&self) -> Result<i64> {
        match self {
            WasmValue::I32(v) => Ok(*v as i64),
            WasmValue::I64(v) => Ok(*v),
            WasmValue::F32(v) => Ok(*v as i64),
            WasmValue::F64(v) => Ok(*v as i64),
            _ => Err(Error::type_error(format!("Cannot convert {:?} to i64", self.value_type()))),
        }
    }
    
    /// 转换为f32
    pub fn as_f32(&self) -> Result<f32> {
        match self {
            WasmValue::I32(v) => Ok(*v as f32),
            WasmValue::I64(v) => Ok(*v as f32),
            WasmValue::F32(v) => Ok(*v),
            WasmValue::F64(v) => Ok(*v as f32),
            _ => Err(Error::type_error(format!("Cannot convert {:?} to f32", self.value_type()))),
        }
    }
    
    /// 转换为f64
    pub fn as_f64(&self) -> Result<f64> {
        match self {
            WasmValue::I32(v) => Ok(*v as f64),
            WasmValue::I64(v) => Ok(*v as f64),
            WasmValue::F32(v) => Ok(*v as f64),
            WasmValue::F64(v) => Ok(*v),
            _ => Err(Error::type_error(format!("Cannot convert {:?} to f64", self.value_type()))),
        }
    }
    
    /// 转换为字符串
    pub fn as_string(&self) -> Result<String> {
        match self {
            WasmValue::String(s) => Ok(s.clone()),
            WasmValue::I32(v) => Ok(v.to_string()),
            WasmValue::I64(v) => Ok(v.to_string()),
            WasmValue::F32(v) => Ok(v.to_string()),
            WasmValue::F64(v) => Ok(v.to_string()),
            WasmValue::Bool(v) => Ok(v.to_string()),
            _ => Err(Error::type_error(format!("Cannot convert {:?} to string", self.value_type()))),
        }
    }
    
    /// 转换为字节数组
    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        match self {
            WasmValue::Bytes(b) => Ok(b.clone()),
            WasmValue::String(s) => Ok(s.as_bytes().to_vec()),
            _ => Err(Error::type_error(format!("Cannot convert {:?} to bytes", self.value_type()))),
        }
    }
    
    /// 序列化为JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::serialization(e.to_string()))
    }
    
    /// 从JSON反序列化
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Error::serialization(e.to_string()))
    }
    
    /// 序列化为二进制
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| Error::serialization(e.to_string()))
    }
    
    /// 从二进制反序列化
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| Error::serialization(e.to_string()))
    }
}

/// WASM类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasmType {
    /// 空类型
    Null,
    /// 布尔类型
    Bool,
    /// 32位整数类型
    I32,
    /// 64位整数类型
    I64,
    /// 32位浮点数类型
    F32,
    /// 64位浮点数类型
    F64,
    /// 字符串类型
    String,
    /// 字节数组类型
    Bytes,
    /// 数组类型
    Array,
    /// 对象类型
    Object,
}

impl std::fmt::Display for WasmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmType::Null => write!(f, "null"),
            WasmType::Bool => write!(f, "bool"),
            WasmType::I32 => write!(f, "i32"),
            WasmType::I64 => write!(f, "i64"),
            WasmType::F32 => write!(f, "f32"),
            WasmType::F64 => write!(f, "f64"),
            WasmType::String => write!(f, "string"),
            WasmType::Bytes => write!(f, "bytes"),
            WasmType::Array => write!(f, "array"),
            WasmType::Object => write!(f, "object"),
        }
    }
}

/// 类型转换器
pub struct WasmTypeConverter;

impl WasmTypeConverter {
    /// 将fdc_core::Value转换为WasmValue
    pub fn from_core_value(value: &Value) -> Result<WasmValue> {
        match value {
            Value::Null => Ok(WasmValue::Null),
            Value::Bool(b) => Ok(WasmValue::Bool(*b)),
            Value::Int8(v) => Ok(WasmValue::I32(*v as i32)),
            Value::Int16(v) => Ok(WasmValue::I32(*v as i32)),
            Value::Int32(v) => Ok(WasmValue::I32(*v)),
            Value::Int64(v) => Ok(WasmValue::I64(*v)),
            Value::Int128(v) => Ok(WasmValue::I64(*v as i64)), // 截断
            Value::UInt8(v) => Ok(WasmValue::I32(*v as i32)),
            Value::UInt16(v) => Ok(WasmValue::I32(*v as i32)),
            Value::UInt32(v) => Ok(WasmValue::I64(*v as i64)),
            Value::UInt64(v) => Ok(WasmValue::I64(*v as i64)), // 可能溢出
            Value::UInt128(v) => Ok(WasmValue::I64(*v as i64)), // 截断
            Value::Float32(v) => Ok(WasmValue::F32(*v)),
            Value::Float64(v) => Ok(WasmValue::F64(*v)),
            Value::String(s) => Ok(WasmValue::String(s.clone())),
            Value::Binary(b) => Ok(WasmValue::Bytes(b.clone())),
            Value::Array(arr) => {
                let wasm_arr: Result<Vec<WasmValue>> = arr.iter()
                    .map(Self::from_core_value)
                    .collect();
                Ok(WasmValue::Array(wasm_arr?))
            }
            Value::Struct(map) => {
                let mut wasm_obj = HashMap::new();
                for (key, val) in map {
                    wasm_obj.insert(key.clone(), Self::from_core_value(val)?);
                }
                Ok(WasmValue::Object(wasm_obj))
            }
            Value::Map(map) => {
                let mut wasm_obj = HashMap::new();
                for (key, val) in map {
                    wasm_obj.insert(key.clone(), Self::from_core_value(val)?);
                }
                Ok(WasmValue::Object(wasm_obj))
            }
            _ => {
                // 对于其他复杂类型，序列化为字节数组
                let bytes = bincode::serialize(value)
                    .map_err(|e| Error::serialization(e.to_string()))?;
                Ok(WasmValue::Bytes(bytes))
            }
        }
    }
    
    /// 将WasmValue转换为fdc_core::Value
    pub fn to_core_value(wasm_value: &WasmValue) -> Result<Value> {
        match wasm_value {
            WasmValue::Null => Ok(Value::Null),
            WasmValue::Bool(b) => Ok(Value::Bool(*b)),
            WasmValue::I32(v) => Ok(Value::Int32(*v)),
            WasmValue::I64(v) => Ok(Value::Int64(*v)),
            WasmValue::F32(v) => Ok(Value::Float32(*v)),
            WasmValue::F64(v) => Ok(Value::Float64(*v)),
            WasmValue::String(s) => Ok(Value::String(s.clone())),
            WasmValue::Bytes(b) => Ok(Value::Binary(b.clone())),
            WasmValue::Array(arr) => {
                let core_arr: Result<Vec<Value>> = arr.iter()
                    .map(Self::to_core_value)
                    .collect();
                Ok(Value::Array(core_arr?))
            }
            WasmValue::Object(obj) => {
                let mut core_map = HashMap::new();
                for (key, val) in obj {
                    core_map.insert(key.clone(), Self::to_core_value(val)?);
                }
                Ok(Value::Struct(core_map))
            }
        }
    }
    
    /// 批量转换
    pub fn from_core_values(values: &[Value]) -> Result<Vec<WasmValue>> {
        values.iter().map(Self::from_core_value).collect()
    }
    
    /// 批量转换
    pub fn to_core_values(wasm_values: &[WasmValue]) -> Result<Vec<Value>> {
        wasm_values.iter().map(Self::to_core_value).collect()
    }
}

/// 函数签名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmFunctionSignature {
    /// 函数名称
    pub name: String,
    /// 输入参数类型
    pub input_types: Vec<WasmType>,
    /// 输出类型
    pub output_type: WasmType,
    /// 函数描述
    pub description: Option<String>,
}

impl WasmFunctionSignature {
    /// 创建新的函数签名
    pub fn new(name: String, input_types: Vec<WasmType>, output_type: WasmType) -> Self {
        Self {
            name,
            input_types,
            output_type,
            description: None,
        }
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// 验证参数类型
    pub fn validate_args(&self, args: &[WasmValue]) -> Result<()> {
        if args.len() != self.input_types.len() {
            return Err(Error::validation(format!(
                "Function {} expects {} arguments, got {}",
                self.name, self.input_types.len(), args.len()
            )));
        }
        
        for (i, (arg, expected_type)) in args.iter().zip(&self.input_types).enumerate() {
            let actual_type = arg.value_type();
            if actual_type != *expected_type {
                return Err(Error::validation(format!(
                    "Function {} argument {} expects type {}, got {}",
                    self.name, i, expected_type, actual_type
                )));
            }
        }
        
        Ok(())
    }
    
    /// 验证返回值类型
    pub fn validate_return(&self, return_value: &WasmValue) -> Result<()> {
        let actual_type = return_value.value_type();
        if actual_type != self.output_type {
            return Err(Error::validation(format!(
                "Function {} expects return type {}, got {}",
                self.name, self.output_type, actual_type
            )));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_value_types() {
        assert_eq!(WasmValue::Null.value_type(), WasmType::Null);
        assert_eq!(WasmValue::Bool(true).value_type(), WasmType::Bool);
        assert_eq!(WasmValue::I32(42).value_type(), WasmType::I32);
        assert_eq!(WasmValue::I64(42).value_type(), WasmType::I64);
        assert_eq!(WasmValue::F32(3.14).value_type(), WasmType::F32);
        assert_eq!(WasmValue::F64(3.14).value_type(), WasmType::F64);
        assert_eq!(WasmValue::String("test".to_string()).value_type(), WasmType::String);
        assert_eq!(WasmValue::Bytes(vec![1, 2, 3]).value_type(), WasmType::Bytes);
    }

    #[test]
    fn test_wasm_value_conversions() {
        let value = WasmValue::I32(42);
        assert_eq!(value.as_i32().unwrap(), 42);
        assert_eq!(value.as_i64().unwrap(), 42);
        assert_eq!(value.as_f32().unwrap(), 42.0);
        assert_eq!(value.as_f64().unwrap(), 42.0);
        assert_eq!(value.as_string().unwrap(), "42");
    }

    #[test]
    fn test_wasm_value_serialization() {
        let value = WasmValue::Object({
            let mut map = HashMap::new();
            map.insert("key".to_string(), WasmValue::String("value".to_string()));
            map
        });
        
        let json = value.to_json().unwrap();
        let deserialized = WasmValue::from_json(&json).unwrap();
        assert_eq!(value, deserialized);
        
        let bytes = value.to_bytes().unwrap();
        let deserialized = WasmValue::from_bytes(&bytes).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn test_type_converter() {
        let core_value = Value::Int32(42);
        let wasm_value = WasmTypeConverter::from_core_value(&core_value).unwrap();
        assert_eq!(wasm_value, WasmValue::I32(42));
        
        let converted_back = WasmTypeConverter::to_core_value(&wasm_value).unwrap();
        assert_eq!(converted_back, core_value);
    }

    #[test]
    fn test_function_signature() {
        let sig = WasmFunctionSignature::new(
            "test_func".to_string(),
            vec![WasmType::I32, WasmType::String],
            WasmType::Bool,
        );
        
        let valid_args = vec![WasmValue::I32(42), WasmValue::String("test".to_string())];
        assert!(sig.validate_args(&valid_args).is_ok());
        
        let invalid_args = vec![WasmValue::I32(42)]; // 缺少参数
        assert!(sig.validate_args(&invalid_args).is_err());
        
        let valid_return = WasmValue::Bool(true);
        assert!(sig.validate_return(&valid_return).is_ok());
        
        let invalid_return = WasmValue::I32(42);
        assert!(sig.validate_return(&invalid_return).is_err());
    }
}
