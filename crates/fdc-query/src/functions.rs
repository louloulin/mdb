//! Built-in functions for query engine

use fdc_core::{error::Result, types::Value};
use std::collections::HashMap;

/// 内置函数
pub struct BuiltinFunctions {
    functions: HashMap<String, fn(&[Value]) -> Result<Value>>,
}

impl BuiltinFunctions {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        
        // 数学函数
        functions.insert("ABS".to_string(), abs as fn(&[Value]) -> Result<Value>);
        functions.insert("ROUND".to_string(), round as fn(&[Value]) -> Result<Value>);
        
        // 字符串函数
        functions.insert("UPPER".to_string(), upper as fn(&[Value]) -> Result<Value>);
        functions.insert("LOWER".to_string(), lower as fn(&[Value]) -> Result<Value>);
        functions.insert("LENGTH".to_string(), length as fn(&[Value]) -> Result<Value>);
        
        Self { functions }
    }
    
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value> {
        if let Some(func) = self.functions.get(&name.to_uppercase()) {
            func(args)
        } else {
            Err(fdc_core::error::Error::validation(format!("Unknown function: {}", name)))
        }
    }
}

impl Default for BuiltinFunctions {
    fn default() -> Self {
        Self::new()
    }
}

// 数学函数实现
fn abs(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(fdc_core::error::Error::validation("ABS requires exactly 1 argument"));
    }
    
    match &args[0] {
        Value::Int32(v) => Ok(Value::Int32(v.abs())),
        Value::Int64(v) => Ok(Value::Int64(v.abs())),
        Value::Float32(v) => Ok(Value::Float32(v.abs())),
        Value::Float64(v) => Ok(Value::Float64(v.abs())),
        _ => Err(fdc_core::error::Error::validation("ABS requires numeric argument")),
    }
}

fn round(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(fdc_core::error::Error::validation("ROUND requires 1 or 2 arguments"));
    }
    
    match &args[0] {
        Value::Float32(v) => Ok(Value::Float32(v.round())),
        Value::Float64(v) => Ok(Value::Float64(v.round())),
        _ => Err(fdc_core::error::Error::validation("ROUND requires numeric argument")),
    }
}

// 字符串函数实现
fn upper(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(fdc_core::error::Error::validation("UPPER requires exactly 1 argument"));
    }
    
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Err(fdc_core::error::Error::validation("UPPER requires string argument")),
    }
}

fn lower(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(fdc_core::error::Error::validation("LOWER requires exactly 1 argument"));
    }
    
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Err(fdc_core::error::Error::validation("LOWER requires string argument")),
    }
}

fn length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(fdc_core::error::Error::validation("LENGTH requires exactly 1 argument"));
    }
    
    match &args[0] {
        Value::String(s) => Ok(Value::Int32(s.len() as i32)),
        _ => Err(fdc_core::error::Error::validation("LENGTH requires string argument")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_functions() {
        let functions = BuiltinFunctions::new();
        
        // 测试ABS函数
        let result = functions.call("ABS", &[Value::Int32(-5)]).unwrap();
        assert_eq!(result, Value::Int32(5));
        
        // 测试UPPER函数
        let result = functions.call("UPPER", &[Value::String("hello".to_string())]).unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
        
        // 测试LENGTH函数
        let result = functions.call("LENGTH", &[Value::String("test".to_string())]).unwrap();
        assert_eq!(result, Value::Int32(4));
    }
}
