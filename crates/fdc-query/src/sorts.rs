//! Sort operations

use fdc_core::{error::Result, types::Value};
use std::collections::HashMap;
use std::cmp::Ordering;

/// 排序操作
pub struct SortOperations;

impl SortOperations {
    pub fn sort(
        mut rows: Vec<HashMap<String, Value>>,
        column: &str,
        ascending: bool,
    ) -> Result<Vec<HashMap<String, Value>>> {
        rows.sort_by(|a, b| {
            let a_val = a.get(column);
            let b_val = b.get(column);
            
            let ordering = match (a_val, b_val) {
                (Some(a), Some(b)) => Self::compare_values(a, b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            };
            
            if ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
        
        Ok(rows)
    }
    
    fn compare_values(a: &Value, b: &Value) -> Ordering {
        match (a, b) {
            (Value::Int32(a), Value::Int32(b)) => a.cmp(b),
            (Value::Int64(a), Value::Int64(b)) => a.cmp(b),
            (Value::Float32(a), Value::Float32(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Float64(a), Value::Float64(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            _ => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_operations() {
        let mut row1 = HashMap::new();
        row1.insert("id".to_string(), Value::Int32(2));
        row1.insert("name".to_string(), Value::String("Bob".to_string()));
        
        let mut row2 = HashMap::new();
        row2.insert("id".to_string(), Value::Int32(1));
        row2.insert("name".to_string(), Value::String("Alice".to_string()));
        
        let rows = vec![row1, row2];
        let result = SortOperations::sort(rows, "id", true).unwrap();
        
        assert_eq!(result[0].get("id"), Some(&Value::Int32(1)));
        assert_eq!(result[1].get("id"), Some(&Value::Int32(2)));
    }
}
