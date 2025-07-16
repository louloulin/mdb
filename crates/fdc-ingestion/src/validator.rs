//! Data validation and quality control

use crate::{config::ValidatorConfig, parser::ParsedData};
use fdc_core::{error::Result, types::Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    /// 必填字段检查
    RequiredFields(Vec<String>),
    /// 数据类型检查
    DataTypes(HashMap<String, String>),
    /// 值范围检查
    ValueRange { field: String, min: f64, max: f64 },
    /// 字符串长度检查
    StringLength { field: String, min: usize, max: usize },
    /// 正则表达式检查
    Regex { field: String, pattern: String },
    /// 自定义验证函数
    Custom { name: String, expression: String },
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否通过验证
    pub is_valid: bool,
    /// 验证错误列表
    pub errors: Vec<ValidationError>,
    /// 验证警告列表
    pub warnings: Vec<ValidationWarning>,
    /// 验证耗时（微秒）
    pub validation_time_us: u64,
    /// 验证时间戳
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

impl ValidationResult {
    /// 创建成功的验证结果
    pub fn success(validation_time_us: u64) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            validation_time_us,
            validated_at: chrono::Utc::now(),
        }
    }
    
    /// 创建失败的验证结果
    pub fn failure(errors: Vec<ValidationError>, validation_time_us: u64) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
            validation_time_us,
            validated_at: chrono::Utc::now(),
        }
    }
    
    /// 添加警告
    pub fn with_warnings(mut self, warnings: Vec<ValidationWarning>) -> Self {
        self.warnings = warnings;
        self
    }
    
    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
    
    /// 获取警告数量
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// 错误类型
    pub error_type: ValidationErrorType,
    /// 字段路径
    pub field_path: String,
    /// 错误消息
    pub message: String,
    /// 期望值
    pub expected: Option<String>,
    /// 实际值
    pub actual: Option<String>,
}

/// 验证错误类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationErrorType {
    /// 缺少必填字段
    MissingRequiredField,
    /// 数据类型不匹配
    TypeMismatch,
    /// 值超出范围
    ValueOutOfRange,
    /// 字符串长度不符
    InvalidStringLength,
    /// 正则表达式不匹配
    RegexMismatch,
    /// 自定义验证失败
    CustomValidationFailed,
}

/// 验证警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// 警告类型
    pub warning_type: String,
    /// 字段路径
    pub field_path: String,
    /// 警告消息
    pub message: String,
}

/// 验证器统计信息
#[derive(Debug, Clone, Default)]
pub struct ValidatorStats {
    /// 验证的消息数
    pub messages_validated: u64,
    /// 验证成功数
    pub validation_successes: u64,
    /// 验证失败数
    pub validation_failures: u64,
    /// 总验证时间（微秒）
    pub total_validation_time_us: u64,
    /// 平均验证时间（微秒）
    pub avg_validation_time_us: f64,
    /// 错误统计
    pub error_counts: HashMap<ValidationErrorType, u64>,
    /// 警告统计
    pub warning_counts: HashMap<String, u64>,
}

impl ValidatorStats {
    /// 记录验证结果
    pub fn record_validation(&mut self, result: &ValidationResult) {
        self.messages_validated += 1;
        self.total_validation_time_us += result.validation_time_us;
        
        if result.is_valid {
            self.validation_successes += 1;
        } else {
            self.validation_failures += 1;
        }
        
        // 统计错误类型
        for error in &result.errors {
            *self.error_counts.entry(error.error_type.clone()).or_insert(0) += 1;
        }
        
        // 统计警告类型
        for warning in &result.warnings {
            *self.warning_counts.entry(warning.warning_type.clone()).or_insert(0) += 1;
        }
        
        self.avg_validation_time_us = self.total_validation_time_us as f64 / self.messages_validated as f64;
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.messages_validated == 0 {
            0.0
        } else {
            self.validation_successes as f64 / self.messages_validated as f64
        }
    }
}

/// 数据验证器
pub struct DataValidator {
    /// 配置
    config: ValidatorConfig,
    /// 验证规则
    rules: Vec<ValidationRule>,
    /// 验证器统计
    stats: Arc<RwLock<ValidatorStats>>,
}

impl DataValidator {
    /// 创建新的数据验证器
    pub fn new(config: ValidatorConfig) -> Self {
        let rules = Self::parse_rules(&config.rules);
        Self {
            config,
            rules,
            stats: Arc::new(RwLock::new(ValidatorStats::default())),
        }
    }
    
    /// 解析验证规则
    fn parse_rules(rule_strings: &[String]) -> Vec<ValidationRule> {
        let mut rules = Vec::new();
        
        for rule_str in rule_strings {
            match rule_str.as_str() {
                "required_fields" => {
                    // 默认必填字段
                    rules.push(ValidationRule::RequiredFields(vec![
                        "timestamp".to_string(),
                        "symbol".to_string(),
                    ]));
                }
                "data_types" => {
                    // 默认数据类型检查
                    let mut types = HashMap::new();
                    types.insert("timestamp".to_string(), "timestamp".to_string());
                    types.insert("price".to_string(), "float64".to_string());
                    types.insert("volume".to_string(), "int64".to_string());
                    rules.push(ValidationRule::DataTypes(types));
                }
                _ => {
                    // 自定义规则
                    rules.push(ValidationRule::Custom {
                        name: rule_str.clone(),
                        expression: rule_str.clone(),
                    });
                }
            }
        }
        
        rules
    }
    
    /// 验证解析后的数据
    pub async fn validate(&self, parsed_data: &ParsedData) -> Result<ValidationResult> {
        if !self.config.enabled {
            return Ok(ValidationResult::success(0));
        }
        
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // 应用所有验证规则
        for rule in &self.rules {
            match self.apply_rule(rule, &parsed_data.value).await {
                Ok(rule_warnings) => {
                    warnings.extend(rule_warnings);
                }
                Err(rule_errors) => {
                    errors.extend(rule_errors);
                }
            }
        }
        
        let validation_time_us = start_time.elapsed().as_micros() as u64;
        
        let result = if errors.is_empty() {
            ValidationResult::success(validation_time_us).with_warnings(warnings)
        } else {
            ValidationResult::failure(errors, validation_time_us).with_warnings(warnings)
        };
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.record_validation(&result);
        }
        
        Ok(result)
    }
    
    /// 应用单个验证规则
    async fn apply_rule(&self, rule: &ValidationRule, value: &Value) -> std::result::Result<Vec<ValidationWarning>, Vec<ValidationError>> {
        match rule {
            ValidationRule::RequiredFields(fields) => {
                self.validate_required_fields(fields, value)
            }
            ValidationRule::DataTypes(types) => {
                self.validate_data_types(types, value)
            }
            ValidationRule::ValueRange { field, min, max } => {
                self.validate_value_range(field, *min, *max, value)
            }
            ValidationRule::StringLength { field, min, max } => {
                self.validate_string_length(field, *min, *max, value)
            }
            ValidationRule::Regex { field, pattern } => {
                self.validate_regex(field, pattern, value)
            }
            ValidationRule::Custom { name, expression: _ } => {
                // 简化的自定义验证
                Ok(vec![ValidationWarning {
                    warning_type: "custom_validation".to_string(),
                    field_path: "root".to_string(),
                    message: format!("Custom validation '{}' not implemented", name),
                }])
            }
        }
    }
    
    /// 验证必填字段
    fn validate_required_fields(&self, fields: &[String], value: &Value) -> std::result::Result<Vec<ValidationWarning>, Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if let Value::Struct(map) = value {
            for field in fields {
                if !map.contains_key(field) {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::MissingRequiredField,
                        field_path: field.clone(),
                        message: format!("Required field '{}' is missing", field),
                        expected: Some("present".to_string()),
                        actual: Some("missing".to_string()),
                    });
                }
            }
        }
        
        if errors.is_empty() {
            Ok(Vec::new())
        } else {
            Err(errors)
        }
    }
    
    /// 验证数据类型
    fn validate_data_types(&self, types: &HashMap<String, String>, value: &Value) -> std::result::Result<Vec<ValidationWarning>, Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if let Value::Struct(map) = value {
            for (field, expected_type) in types {
                if let Some(field_value) = map.get(field) {
                    let actual_type = self.get_value_type_name(field_value);
                    if &actual_type != expected_type {
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::TypeMismatch,
                            field_path: field.clone(),
                            message: format!("Field '{}' has wrong type", field),
                            expected: Some(expected_type.clone()),
                            actual: Some(actual_type),
                        });
                    }
                }
            }
        }
        
        if errors.is_empty() {
            Ok(Vec::new())
        } else {
            Err(errors)
        }
    }
    
    /// 获取值的类型名称
    fn get_value_type_name(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "bool".to_string(),
            Value::Int8(_) => "int8".to_string(),
            Value::Int16(_) => "int16".to_string(),
            Value::Int32(_) => "int32".to_string(),
            Value::Int64(_) => "int64".to_string(),
            Value::Int128(_) => "int128".to_string(),
            Value::UInt8(_) => "uint8".to_string(),
            Value::UInt16(_) => "uint16".to_string(),
            Value::UInt32(_) => "uint32".to_string(),
            Value::UInt64(_) => "uint64".to_string(),
            Value::UInt128(_) => "uint128".to_string(),
            Value::Float32(_) => "float32".to_string(),
            Value::Float64(_) => "float64".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Binary(_) => "binary".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Struct(_) => "struct".to_string(),
            Value::Map(_) => "map".to_string(),
            _ => "unknown".to_string(),
        }
    }
    
    /// 验证值范围
    fn validate_value_range(&self, _field: &str, _min: f64, _max: f64, _value: &Value) -> std::result::Result<Vec<ValidationWarning>, Vec<ValidationError>> {
        // 简化实现
        Ok(Vec::new())
    }
    
    /// 验证字符串长度
    fn validate_string_length(&self, _field: &str, _min: usize, _max: usize, _value: &Value) -> std::result::Result<Vec<ValidationWarning>, Vec<ValidationError>> {
        // 简化实现
        Ok(Vec::new())
    }

    /// 验证正则表达式
    fn validate_regex(&self, _field: &str, _pattern: &str, _value: &Value) -> std::result::Result<Vec<ValidationWarning>, Vec<ValidationError>> {
        // 简化实现
        Ok(Vec::new())
    }
    
    /// 获取验证器统计信息
    pub async fn get_stats(&self) -> ValidatorStats {
        self.stats.read().await.clone()
    }
    
    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ValidatorStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult::success(1000);
        assert!(result.is_valid);
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.validation_time_us, 1000);
        
        let errors = vec![ValidationError {
            error_type: ValidationErrorType::MissingRequiredField,
            field_path: "test".to_string(),
            message: "Test error".to_string(),
            expected: None,
            actual: None,
        }];
        
        let result = ValidationResult::failure(errors, 2000);
        assert!(!result.is_valid);
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.validation_time_us, 2000);
    }

    #[tokio::test]
    async fn test_required_fields_validation() {
        let config = ValidatorConfig {
            enabled: true,
            rules: vec!["required_fields".to_string()],
            ..Default::default()
        };
        
        let validator = DataValidator::new(config);
        
        // 测试缺少必填字段的情况
        let mut map = HashMap::new();
        map.insert("price".to_string(), Value::Float64(100.0));
        let value = Value::Struct(map);
        
        let parsed_data = crate::parser::ParsedData::new(
            value,
            "test".to_string(),
            100,
            1000,
        );
        
        let result = validator.validate(&parsed_data).await.unwrap();
        assert!(!result.is_valid);
        assert!(result.error_count() > 0);
    }

    #[test]
    fn test_validator_stats() {
        let mut stats = ValidatorStats::default();
        
        let success_result = ValidationResult::success(1000);
        let failure_result = ValidationResult::failure(vec![ValidationError {
            error_type: ValidationErrorType::MissingRequiredField,
            field_path: "test".to_string(),
            message: "Test".to_string(),
            expected: None,
            actual: None,
        }], 2000);
        
        stats.record_validation(&success_result);
        stats.record_validation(&failure_result);
        
        assert_eq!(stats.messages_validated, 2);
        assert_eq!(stats.validation_successes, 1);
        assert_eq!(stats.validation_failures, 1);
        assert_eq!(stats.success_rate(), 0.5);
    }
}
