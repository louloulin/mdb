//! Filter operations

use fdc_core::{error::Result, types::Value};
use std::collections::HashMap;

/// 过滤操作
pub struct FilterOperations;

impl FilterOperations {
    pub fn apply_filter(
        rows: &[HashMap<String, Value>],
        column: &str,
        operator: &str,
        value: &Value,
    ) -> Result<Vec<HashMap<String, Value>>> {
        let mut result = Vec::new();
        
        for row in rows {
            if let Some(row_value) = row.get(column) {
                if Self::evaluate_condition(row_value, operator, value)? {
                    result.push(row.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    fn evaluate_condition(left: &Value, operator: &str, right: &Value) -> Result<bool> {
        match operator {
            "=" => Ok(left == right),
            "!=" => Ok(left != right),
            "<" => Ok(left < right),
            "<=" => Ok(left <= right),
            ">" => Ok(left > right),
            ">=" => Ok(left >= right),
            _ => Err(fdc_core::error::Error::validation(format!("Unknown operator: {}", operator))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_operations() {
        let mut row1 = HashMap::new();
        row1.insert("id".to_string(), Value::Int32(1));
        row1.insert("age".to_string(), Value::Int32(25));
        
        let mut row2 = HashMap::new();
        row2.insert("id".to_string(), Value::Int32(2));
        row2.insert("age".to_string(), Value::Int32(30));
        
        let rows = vec![row1, row2];
        let result = FilterOperations::apply_filter(&rows, "age", ">", &Value::Int32(26)).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("id"), Some(&Value::Int32(2)));
    }
}
