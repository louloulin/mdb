//! Type definition system

use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 类型种类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeKind {
    /// 基础类型
    Primitive(PrimitiveType),
    /// 结构体类型
    Struct,
    /// 枚举类型
    Enum,
    /// 联合类型
    Union,
    /// 数组类型
    Array(Box<TypeDefinition>),
    /// 映射类型
    Map(Box<TypeDefinition>, Box<TypeDefinition>),
    /// 可选类型
    Optional(Box<TypeDefinition>),
    /// 引用类型
    Reference(String),
    /// 函数类型
    Function {
        inputs: Vec<TypeDefinition>,
        output: Box<TypeDefinition>,
    },
    /// 自定义类型
    Custom(String),
}

/// 基础类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    Bool,
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    F32, F64,
    String,
    Bytes,
    Timestamp,
    Decimal,
    BigInt,
}

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::I8 => write!(f, "i8"),
            PrimitiveType::I16 => write!(f, "i16"),
            PrimitiveType::I32 => write!(f, "i32"),
            PrimitiveType::I64 => write!(f, "i64"),
            PrimitiveType::I128 => write!(f, "i128"),
            PrimitiveType::U8 => write!(f, "u8"),
            PrimitiveType::U16 => write!(f, "u16"),
            PrimitiveType::U32 => write!(f, "u32"),
            PrimitiveType::U64 => write!(f, "u64"),
            PrimitiveType::U128 => write!(f, "u128"),
            PrimitiveType::F32 => write!(f, "f32"),
            PrimitiveType::F64 => write!(f, "f64"),
            PrimitiveType::String => write!(f, "string"),
            PrimitiveType::Bytes => write!(f, "bytes"),
            PrimitiveType::Timestamp => write!(f, "timestamp"),
            PrimitiveType::Decimal => write!(f, "decimal"),
            PrimitiveType::BigInt => write!(f, "bigint"),
        }
    }
}

/// 类型定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDefinition {
    /// 类型ID
    pub id: Uuid,
    /// 类型名称
    pub name: String,
    /// 类型种类
    pub kind: TypeKind,
    /// 类型描述
    pub description: Option<String>,
    /// 类型版本
    pub version: String,
    /// 字段定义（仅适用于结构体、枚举等）
    pub fields: Vec<FieldDefinition>,
    /// 类型约束
    pub constraints: Vec<TypeConstraint>,
    /// 类型属性
    pub attributes: HashMap<String, String>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 字段定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// 字段名称
    pub name: String,
    /// 字段类型
    pub field_type: TypeDefinition,
    /// 是否可选
    pub optional: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 字段描述
    pub description: Option<String>,
    /// 字段约束
    pub constraints: Vec<FieldConstraint>,
    /// 字段属性
    pub attributes: HashMap<String, String>,
}

/// 类型约束
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeConstraint {
    /// 最小值约束
    MinValue(f64),
    /// 最大值约束
    MaxValue(f64),
    /// 最小长度约束
    MinLength(usize),
    /// 最大长度约束
    MaxLength(usize),
    /// 正则表达式约束
    Pattern(String),
    /// 枚举值约束
    Enum(Vec<String>),
    /// 自定义约束
    Custom {
        name: String,
        expression: String,
    },
}

/// 字段约束
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldConstraint {
    /// 必填字段
    Required,
    /// 唯一字段
    Unique,
    /// 索引字段
    Indexed,
    /// 不可变字段
    Immutable,
    /// 自定义约束
    Custom {
        name: String,
        expression: String,
    },
}

impl TypeDefinition {
    /// 创建新的类型定义
    pub fn new(name: String, kind: TypeKind) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            kind,
            description: None,
            version: "1.0.0".to_string(),
            fields: Vec::new(),
            constraints: Vec::new(),
            attributes: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = chrono::Utc::now();
        self
    }
    
    /// 设置版本
    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self.updated_at = chrono::Utc::now();
        self
    }
    
    /// 添加字段
    pub fn add_field(&mut self, field: FieldDefinition) {
        self.fields.push(field);
        self.updated_at = chrono::Utc::now();
    }
    
    /// 添加约束
    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
        self.updated_at = chrono::Utc::now();
    }
    
    /// 设置属性
    pub fn set_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
        self.updated_at = chrono::Utc::now();
    }
    
    /// 获取字段
    pub fn get_field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields.iter().find(|f| f.name == name)
    }
    
    /// 检查是否为基础类型
    pub fn is_primitive(&self) -> bool {
        matches!(self.kind, TypeKind::Primitive(_))
    }
    
    /// 检查是否为复合类型
    pub fn is_composite(&self) -> bool {
        matches!(self.kind, TypeKind::Struct | TypeKind::Enum | TypeKind::Union)
    }
    
    /// 检查是否为容器类型
    pub fn is_container(&self) -> bool {
        matches!(self.kind, TypeKind::Array(_) | TypeKind::Map(_, _))
    }
    
    /// 获取类型大小（字节）
    pub fn size_hint(&self) -> Option<usize> {
        match &self.kind {
            TypeKind::Primitive(prim) => Some(match prim {
                PrimitiveType::Bool => 1,
                PrimitiveType::I8 | PrimitiveType::U8 => 1,
                PrimitiveType::I16 | PrimitiveType::U16 => 2,
                PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::F32 => 4,
                PrimitiveType::I64 | PrimitiveType::U64 | PrimitiveType::F64 => 8,
                PrimitiveType::I128 | PrimitiveType::U128 => 16,
                PrimitiveType::Timestamp => 8,
                _ => return None, // 动态大小
            }),
            TypeKind::Struct => {
                let mut total = 0;
                for field in &self.fields {
                    if let Some(size) = field.field_type.size_hint() {
                        total += size;
                    } else {
                        return None; // 包含动态大小字段
                    }
                }
                Some(total)
            }
            _ => None, // 动态大小或未知
        }
    }
    
    /// 验证类型定义
    pub fn validate(&self) -> Result<()> {
        // 检查名称
        if self.name.is_empty() {
            return Err(Error::validation("Type name cannot be empty"));
        }
        
        // 检查版本格式
        if self.version.is_empty() {
            return Err(Error::validation("Type version cannot be empty"));
        }
        
        // 验证字段
        for field in &self.fields {
            field.validate()?;
        }
        
        // 验证约束
        for constraint in &self.constraints {
            constraint.validate()?;
        }
        
        // 检查循环引用
        self.check_circular_references()?;
        
        Ok(())
    }
    
    /// 检查循环引用
    fn check_circular_references(&self) -> Result<()> {
        let mut visited = std::collections::HashSet::new();
        self.check_circular_references_recursive(&mut visited)
    }
    
    fn check_circular_references_recursive(
        &self,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if visited.contains(&self.name) {
            return Err(Error::validation(format!(
                "Circular reference detected in type: {}",
                self.name
            )));
        }
        
        visited.insert(self.name.clone());
        
        for field in &self.fields {
            field.field_type.check_circular_references_recursive(visited)?;
        }
        
        visited.remove(&self.name);
        Ok(())
    }
}

impl FieldDefinition {
    /// 创建新的字段定义
    pub fn new(name: String, field_type: TypeDefinition) -> Self {
        Self {
            name,
            field_type,
            optional: false,
            default_value: None,
            description: None,
            constraints: Vec::new(),
            attributes: HashMap::new(),
        }
    }
    
    /// 设置为可选字段
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    
    /// 设置默认值
    pub fn with_default(mut self, default_value: String) -> Self {
        self.default_value = Some(default_value);
        self
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// 添加约束
    pub fn add_constraint(&mut self, constraint: FieldConstraint) {
        self.constraints.push(constraint);
    }
    
    /// 验证字段定义
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::validation("Field name cannot be empty"));
        }
        
        self.field_type.validate()?;
        
        Ok(())
    }
}

impl TypeConstraint {
    /// 验证约束
    pub fn validate(&self) -> Result<()> {
        match self {
            TypeConstraint::MinValue(min) => {
                if min.is_nan() || min.is_infinite() {
                    return Err(Error::validation("MinValue constraint must be finite"));
                }
            }
            TypeConstraint::MaxValue(max) => {
                if max.is_nan() || max.is_infinite() {
                    return Err(Error::validation("MaxValue constraint must be finite"));
                }
            }
            TypeConstraint::Pattern(pattern) => {
                regex::Regex::new(pattern)
                    .map_err(|e| Error::validation(format!("Invalid regex pattern: {}", e)))?;
            }
            TypeConstraint::Enum(values) => {
                if values.is_empty() {
                    return Err(Error::validation("Enum constraint cannot be empty"));
                }
            }
            TypeConstraint::Custom { name, expression } => {
                if name.is_empty() {
                    return Err(Error::validation("Custom constraint name cannot be empty"));
                }
                if expression.is_empty() {
                    return Err(Error::validation("Custom constraint expression cannot be empty"));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_type_display() {
        assert_eq!(PrimitiveType::Bool.to_string(), "bool");
        assert_eq!(PrimitiveType::I32.to_string(), "i32");
        assert_eq!(PrimitiveType::String.to_string(), "string");
    }

    #[test]
    fn test_type_definition_creation() {
        let type_def = TypeDefinition::new(
            "TestType".to_string(),
            TypeKind::Primitive(PrimitiveType::I32),
        );
        
        assert_eq!(type_def.name, "TestType");
        assert_eq!(type_def.version, "1.0.0");
        assert!(type_def.fields.is_empty());
        assert!(type_def.constraints.is_empty());
    }

    #[test]
    fn test_field_definition() {
        let field_type = TypeDefinition::new(
            "String".to_string(),
            TypeKind::Primitive(PrimitiveType::String),
        );
        
        let field = FieldDefinition::new("name".to_string(), field_type)
            .optional()
            .with_default("default_name".to_string())
            .with_description("Name field".to_string());
        
        assert_eq!(field.name, "name");
        assert!(field.optional);
        assert_eq!(field.default_value, Some("default_name".to_string()));
        assert_eq!(field.description, Some("Name field".to_string()));
    }

    #[test]
    fn test_type_size_hint() {
        let bool_type = TypeDefinition::new(
            "Bool".to_string(),
            TypeKind::Primitive(PrimitiveType::Bool),
        );
        assert_eq!(bool_type.size_hint(), Some(1));
        
        let i64_type = TypeDefinition::new(
            "I64".to_string(),
            TypeKind::Primitive(PrimitiveType::I64),
        );
        assert_eq!(i64_type.size_hint(), Some(8));
        
        let string_type = TypeDefinition::new(
            "String".to_string(),
            TypeKind::Primitive(PrimitiveType::String),
        );
        assert_eq!(string_type.size_hint(), None);
    }

    #[test]
    fn test_type_validation() {
        let mut type_def = TypeDefinition::new(
            "TestType".to_string(),
            TypeKind::Primitive(PrimitiveType::I32),
        );
        
        assert!(type_def.validate().is_ok());
        
        // 测试空名称
        type_def.name = String::new();
        assert!(type_def.validate().is_err());
    }

    #[test]
    fn test_constraint_validation() {
        let min_constraint = TypeConstraint::MinValue(10.0);
        assert!(min_constraint.validate().is_ok());
        
        let invalid_min = TypeConstraint::MinValue(f64::NAN);
        assert!(invalid_min.validate().is_err());
        
        let pattern_constraint = TypeConstraint::Pattern(r"^\d+$".to_string());
        assert!(pattern_constraint.validate().is_ok());
        
        let invalid_pattern = TypeConstraint::Pattern("[".to_string());
        assert!(invalid_pattern.validate().is_err());
    }
}
