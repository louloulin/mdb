//! Projection operations

use fdc_core::{error::Result, types::Value};
use std::collections::HashMap;

/// 投影操作
pub struct ProjectionOperations;

impl ProjectionOperations {
    pub fn project(
        rows: &[HashMap<String, Value>],
        columns: &[String],
    ) -> Result<Vec<HashMap<String, Value>>> {
        let mut result = Vec::new();
        
        for row in rows {
            let mut projected_row = HashMap::new();
            
            for column in columns {
                if column == "*" {
                    // 选择所有列
                    projected_row = row.clone();
                    break;
                } else if let Some(value) = row.get(column) {
                    projected_row.insert(column.clone(), value.clone());
                }
            }
            
            result.push(projected_row);
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection() {
        let mut row = HashMap::new();
        row.insert("id".to_string(), Value::Int32(1));
        row.insert("name".to_string(), Value::String("Alice".to_string()));
        row.insert("age".to_string(), Value::Int32(25));
        
        let rows = vec![row];
        let columns = vec!["id".to_string(), "name".to_string()];
        
        let result = ProjectionOperations::project(&rows, &columns).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].contains_key("id"));
        assert!(result[0].contains_key("name"));
        assert!(!result[0].contains_key("age"));
    }
}
