//! Aggregate functions

use fdc_core::{error::Result, types::Value};
use serde::{Deserialize, Serialize};

/// 聚合函数类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    First,
    Last,
}

impl AggregateFunction {
    pub fn apply(&self, values: &[Value]) -> Result<Value> {
        match self {
            AggregateFunction::Count => Ok(Value::Int64(values.len() as i64)),
            AggregateFunction::Sum => self.sum(values),
            AggregateFunction::Avg => self.avg(values),
            AggregateFunction::Min => self.min(values),
            AggregateFunction::Max => self.max(values),
            AggregateFunction::First => values.first().cloned().ok_or_else(|| 
                fdc_core::error::Error::validation("No values for FIRST aggregate")),
            AggregateFunction::Last => values.last().cloned().ok_or_else(|| 
                fdc_core::error::Error::validation("No values for LAST aggregate")),
        }
    }
    
    fn sum(&self, values: &[Value]) -> Result<Value> {
        let mut sum = 0.0;
        for value in values {
            match value {
                Value::Int32(v) => sum += *v as f64,
                Value::Int64(v) => sum += *v as f64,
                Value::Float32(v) => sum += *v as f64,
                Value::Float64(v) => sum += *v,
                _ => return Err(fdc_core::error::Error::validation("SUM requires numeric values")),
            }
        }
        Ok(Value::Float64(sum))
    }
    
    fn avg(&self, values: &[Value]) -> Result<Value> {
        if values.is_empty() {
            return Err(fdc_core::error::Error::validation("AVG requires at least one value"));
        }
        
        let sum = self.sum(values)?;
        if let Value::Float64(sum_val) = sum {
            Ok(Value::Float64(sum_val / values.len() as f64))
        } else {
            Err(fdc_core::error::Error::validation("AVG calculation error"))
        }
    }
    
    fn min(&self, values: &[Value]) -> Result<Value> {
        values.iter().min().cloned().ok_or_else(|| 
            fdc_core::error::Error::validation("No values for MIN aggregate"))
    }
    
    fn max(&self, values: &[Value]) -> Result<Value> {
        values.iter().max().cloned().ok_or_else(|| 
            fdc_core::error::Error::validation("No values for MAX aggregate"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_aggregate() {
        let values = vec![Value::Int32(1), Value::Int32(2), Value::Int32(3)];
        let result = AggregateFunction::Count.apply(&values).unwrap();
        assert_eq!(result, Value::Int64(3));
    }

    #[test]
    fn test_sum_aggregate() {
        let values = vec![Value::Int32(1), Value::Int32(2), Value::Int32(3)];
        let result = AggregateFunction::Sum.apply(&values).unwrap();
        assert_eq!(result, Value::Float64(6.0));
    }
}
