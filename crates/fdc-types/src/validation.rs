//! Type validation system

use crate::definition::{TypeDefinition, TypeConstraint, FieldConstraint};
use fdc_core::{error::{Error, Result}, types::Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field_path: String,
    pub error_type: ValidationErrorType,
    pub message: String,
}

/// 验证错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationErrorType {
    TypeMismatch,
    ConstraintViolation,
    RequiredFieldMissing,
    InvalidFormat,
    OutOfRange,
    Custom(String),
}

/// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Required,
    MinValue(f64),
    MaxValue(f64),
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Custom { name: String, expression: String },
}

/// 类型验证器
pub struct TypeValidator {
    custom_validators: HashMap<String, Box<dyn Fn(&Value) -> Result<()> + Send + Sync>>,
}

impl TypeValidator {
    pub fn new() -> Self {
        Self {
            custom_validators: HashMap::new(),
        }
    }
    
    /// 验证值是否符合类型定义
    pub fn validate_value(&self, value: &Value, type_def: &TypeDefinition) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();
        self.validate_value_recursive(value, type_def, "", &mut errors);
        Ok(errors)
    }
    
    fn validate_value_recursive(
        &self,
        value: &Value,
        type_def: &TypeDefinition,
        field_path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        // 验证类型约束
        for constraint in &type_def.constraints {
            if let Err(e) = self.validate_constraint(value, constraint) {
                errors.push(ValidationError {
                    field_path: field_path.to_string(),
                    error_type: ValidationErrorType::ConstraintViolation,
                    message: e.to_string(),
                });
            }
        }
    }
    
    fn validate_constraint(&self, value: &Value, constraint: &TypeConstraint) -> Result<()> {
        match constraint {
            TypeConstraint::MinValue(min) => {
                if let Some(num_val) = self.extract_numeric_value(value) {
                    if num_val < *min {
                        return Err(Error::validation(format!("Value {} is less than minimum {}", num_val, min)));
                    }
                }
            }
            TypeConstraint::MaxValue(max) => {
                if let Some(num_val) = self.extract_numeric_value(value) {
                    if num_val > *max {
                        return Err(Error::validation(format!("Value {} is greater than maximum {}", num_val, max)));
                    }
                }
            }
            TypeConstraint::Pattern(pattern) => {
                if let Value::String(s) = value {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?;
                    if !regex.is_match(s) {
                        return Err(Error::validation(format!("String '{}' does not match pattern '{}'", s, pattern)));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn extract_numeric_value(&self, value: &Value) -> Option<f64> {
        match value {
            Value::Int32(v) => Some(*v as f64),
            Value::Int64(v) => Some(*v as f64),
            Value::Float32(v) => Some(*v as f64),
            Value::Float64(v) => Some(*v),
            _ => None,
        }
    }
}

impl Default for TypeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{TypeDefinition, TypeKind, PrimitiveType};

    #[test]
    fn test_validation() {
        let validator = TypeValidator::new();
        let mut type_def = TypeDefinition::new("test".to_string(), TypeKind::Primitive(PrimitiveType::I32));
        type_def.add_constraint(TypeConstraint::MinValue(0.0));
        type_def.add_constraint(TypeConstraint::MaxValue(100.0));
        
        let valid_value = Value::Int32(50);
        let errors = validator.validate_value(&valid_value, &type_def).unwrap();
        assert!(errors.is_empty());
        
        let invalid_value = Value::Int32(-10);
        let errors = validator.validate_value(&invalid_value, &type_def).unwrap();
        assert!(!errors.is_empty());
    }
}
