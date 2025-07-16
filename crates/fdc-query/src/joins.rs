//! Join operations

use fdc_core::{error::Result, types::Value};
use std::collections::HashMap;

/// 连接操作
pub struct JoinOperations;

impl JoinOperations {
    pub fn inner_join(
        left: &[HashMap<String, Value>],
        right: &[HashMap<String, Value>],
        left_key: &str,
        right_key: &str,
    ) -> Result<Vec<HashMap<String, Value>>> {
        let mut result = Vec::new();
        
        for left_row in left {
            if let Some(left_value) = left_row.get(left_key) {
                for right_row in right {
                    if let Some(right_value) = right_row.get(right_key) {
                        if left_value == right_value {
                            let mut joined_row = left_row.clone();
                            for (key, value) in right_row {
                                joined_row.insert(format!("right_{}", key), value.clone());
                            }
                            result.push(joined_row);
                        }
                    }
                }
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inner_join() {
        let mut left_row = HashMap::new();
        left_row.insert("id".to_string(), Value::Int32(1));
        left_row.insert("name".to_string(), Value::String("Alice".to_string()));
        
        let mut right_row = HashMap::new();
        right_row.insert("user_id".to_string(), Value::Int32(1));
        right_row.insert("order_id".to_string(), Value::Int32(100));
        
        let left = vec![left_row];
        let right = vec![right_row];
        
        let result = JoinOperations::inner_join(&left, &right, "id", "user_id").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].contains_key("name"));
        assert!(result[0].contains_key("right_order_id"));
    }
}
