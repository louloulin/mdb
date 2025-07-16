//! WASM type integration

use crate::definition::TypeDefinition;
use fdc_wasm::types::{WasmValue, WasmType};
use fdc_core::{error::Result, types::Value};

/// WASM类型集成
pub struct WasmTypeIntegration;

impl WasmTypeIntegration {
    pub fn new() -> Self {
        Self
    }
}

/// WASM类型转换器
pub struct WasmTypeConverter;

impl WasmTypeConverter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn to_wasm_value(&self, value: &Value) -> Result<WasmValue> {
        match value {
            Value::Bool(b) => Ok(WasmValue::Bool(*b)),
            Value::Int32(i) => Ok(WasmValue::I32(*i)),
            Value::Int64(i) => Ok(WasmValue::I64(*i)),
            Value::Float32(f) => Ok(WasmValue::F32(*f)),
            Value::Float64(f) => Ok(WasmValue::F64(*f)),
            Value::String(s) => Ok(WasmValue::String(s.clone())),
            _ => Ok(WasmValue::Null),
        }
    }
    
    pub fn from_wasm_value(&self, wasm_value: &WasmValue) -> Result<Value> {
        match wasm_value {
            WasmValue::Bool(b) => Ok(Value::Bool(*b)),
            WasmValue::I32(i) => Ok(Value::Int32(*i)),
            WasmValue::I64(i) => Ok(Value::Int64(*i)),
            WasmValue::F32(f) => Ok(Value::Float32(*f)),
            WasmValue::F64(f) => Ok(Value::Float64(*f)),
            WasmValue::String(s) => Ok(Value::String(s.clone())),
            _ => Ok(Value::Null),
        }
    }
}

impl Default for WasmTypeConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_conversion() {
        let converter = WasmTypeConverter::new();
        let value = Value::Int32(42);
        let wasm_value = converter.to_wasm_value(&value).unwrap();
        assert_eq!(wasm_value, WasmValue::I32(42));
        
        let converted_back = converter.from_wasm_value(&wasm_value).unwrap();
        assert_eq!(converted_back, value);
    }
}
