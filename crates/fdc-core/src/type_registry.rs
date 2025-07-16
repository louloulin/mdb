//! Type registry for Financial Data Center

use crate::error::{Error, Result};
use crate::types::{TypeId, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// 类型注册表
#[derive(Debug)]
pub struct TypeRegistry {
    basic_types: Arc<RwLock<HashMap<TypeId, BasicTypeInfo>>>,
    composite_types: Arc<RwLock<HashMap<TypeId, CompositeTypeInfo>>>,
    user_types: Arc<RwLock<HashMap<TypeId, UserTypeInfo>>>,
    wasm_types: Arc<RwLock<HashMap<TypeId, WasmTypeInfo>>>,
    type_name_to_id: Arc<RwLock<HashMap<String, TypeId>>>,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistry {
    /// 创建新的类型注册表
    pub fn new() -> Self {
        let registry = Self {
            basic_types: Arc::new(RwLock::new(HashMap::new())),
            composite_types: Arc::new(RwLock::new(HashMap::new())),
            user_types: Arc::new(RwLock::new(HashMap::new())),
            wasm_types: Arc::new(RwLock::new(HashMap::new())),
            type_name_to_id: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // 注册基础类型
        registry.register_builtin_types();
        registry
    }
    
    /// 注册基础类型
    fn register_builtin_types(&self) {
        let basic_types = [
            ("null", BasicTypeKind::Null),
            ("bool", BasicTypeKind::Bool),
            ("i8", BasicTypeKind::Int8),
            ("i16", BasicTypeKind::Int16),
            ("i32", BasicTypeKind::Int32),
            ("i64", BasicTypeKind::Int64),
            ("i128", BasicTypeKind::Int128),
            ("u8", BasicTypeKind::UInt8),
            ("u16", BasicTypeKind::UInt16),
            ("u32", BasicTypeKind::UInt32),
            ("u64", BasicTypeKind::UInt64),
            ("u128", BasicTypeKind::UInt128),
            ("f32", BasicTypeKind::Float32),
            ("f64", BasicTypeKind::Float64),
            ("string", BasicTypeKind::String),
            ("binary", BasicTypeKind::Binary),
            ("timestamp", BasicTypeKind::Timestamp),
            ("price", BasicTypeKind::Price),
            ("volume", BasicTypeKind::Volume),
            ("symbol", BasicTypeKind::Symbol),
            ("exchange_id", BasicTypeKind::ExchangeId),
        ];
        
        for (name, kind) in &basic_types {
            let type_id = TypeId::new();
            let type_info = BasicTypeInfo {
                type_id,
                name: name.to_string(),
                kind: *kind,
                size_bytes: Self::get_basic_type_size(*kind),
                alignment: Self::get_basic_type_alignment(*kind),
            };
            
            self.basic_types.write().insert(type_id, type_info);
            self.type_name_to_id.write().insert(name.to_string(), type_id);
        }
    }
    
    /// 获取基础类型大小
    fn get_basic_type_size(kind: BasicTypeKind) -> usize {
        match kind {
            BasicTypeKind::Null => 0,
            BasicTypeKind::Bool => 1,
            BasicTypeKind::Int8 | BasicTypeKind::UInt8 => 1,
            BasicTypeKind::Int16 | BasicTypeKind::UInt16 => 2,
            BasicTypeKind::Int32 | BasicTypeKind::UInt32 | BasicTypeKind::Float32 => 4,
            BasicTypeKind::Int64 | BasicTypeKind::UInt64 | BasicTypeKind::Float64 => 8,
            BasicTypeKind::Int128 | BasicTypeKind::UInt128 => 16,
            BasicTypeKind::String | BasicTypeKind::Binary => std::mem::size_of::<Vec<u8>>(),
            BasicTypeKind::Timestamp => 8,
            BasicTypeKind::Price => 16, // Decimal128
            BasicTypeKind::Volume => 8,
            BasicTypeKind::Symbol => std::mem::size_of::<String>(),
            BasicTypeKind::ExchangeId => 2,
        }
    }
    
    /// 获取基础类型对齐
    fn get_basic_type_alignment(kind: BasicTypeKind) -> usize {
        match kind {
            BasicTypeKind::Null => 1,
            BasicTypeKind::Bool => 1,
            BasicTypeKind::Int8 | BasicTypeKind::UInt8 => 1,
            BasicTypeKind::Int16 | BasicTypeKind::UInt16 => 2,
            BasicTypeKind::Int32 | BasicTypeKind::UInt32 | BasicTypeKind::Float32 => 4,
            BasicTypeKind::Int64 | BasicTypeKind::UInt64 | BasicTypeKind::Float64 => 8,
            BasicTypeKind::Int128 | BasicTypeKind::UInt128 => 16,
            BasicTypeKind::String | BasicTypeKind::Binary => 8,
            BasicTypeKind::Timestamp => 8,
            BasicTypeKind::Price => 16,
            BasicTypeKind::Volume => 8,
            BasicTypeKind::Symbol => 8,
            BasicTypeKind::ExchangeId => 2,
        }
    }
    
    /// 注册用户定义类型
    pub fn register_user_type(&self, type_info: UserTypeInfo) -> Result<()> {
        let type_id = type_info.type_id;
        let name = type_info.name.clone();
        
        // 检查名称是否已存在
        if self.type_name_to_id.read().contains_key(&name) {
            return Err(Error::already_exists(format!("Type name: {}", name)));
        }
        
        self.user_types.write().insert(type_id, type_info);
        self.type_name_to_id.write().insert(name, type_id);
        
        Ok(())
    }
    
    /// 注册WASM类型
    pub fn register_wasm_type(&self, type_info: WasmTypeInfo) -> Result<()> {
        let type_id = type_info.type_id;
        let name = type_info.name.clone();
        
        if self.type_name_to_id.read().contains_key(&name) {
            return Err(Error::already_exists(format!("Type name: {}", name)));
        }
        
        self.wasm_types.write().insert(type_id, type_info);
        self.type_name_to_id.write().insert(name, type_id);
        
        Ok(())
    }
    
    /// 根据名称获取类型ID
    pub fn get_type_id(&self, name: &str) -> Option<TypeId> {
        self.type_name_to_id.read().get(name).copied()
    }
    
    /// 根据类型ID获取类型信息
    pub fn get_type_info(&self, type_id: TypeId) -> Option<TypeInfo> {
        // 检查基础类型
        if let Some(info) = self.basic_types.read().get(&type_id) {
            return Some(TypeInfo::Basic(info.clone()));
        }
        
        // 检查复合类型
        if let Some(info) = self.composite_types.read().get(&type_id) {
            return Some(TypeInfo::Composite(info.clone()));
        }
        
        // 检查用户类型
        if let Some(info) = self.user_types.read().get(&type_id) {
            return Some(TypeInfo::User(info.clone()));
        }
        
        // 检查WASM类型
        if let Some(info) = self.wasm_types.read().get(&type_id) {
            return Some(TypeInfo::Wasm(info.clone()));
        }
        
        None
    }
    
    /// 验证值是否符合类型
    pub fn validate_value(&self, value: &Value, type_id: TypeId) -> Result<()> {
        let type_info = self.get_type_info(type_id)
            .ok_or_else(|| Error::not_found(format!("Type ID: {:?}", type_id)))?;
        
        match type_info {
            TypeInfo::Basic(info) => self.validate_basic_value(value, &info),
            TypeInfo::Composite(info) => self.validate_composite_value(value, &info),
            TypeInfo::User(info) => self.validate_user_value(value, &info),
            TypeInfo::Wasm(info) => self.validate_wasm_value(value, &info),
        }
    }
    
    /// 验证基础类型值
    fn validate_basic_value(&self, value: &Value, info: &BasicTypeInfo) -> Result<()> {
        let matches = match (value, info.kind) {
            (Value::Null, BasicTypeKind::Null) => true,
            (Value::Bool(_), BasicTypeKind::Bool) => true,
            (Value::Int8(_), BasicTypeKind::Int8) => true,
            (Value::Int16(_), BasicTypeKind::Int16) => true,
            (Value::Int32(_), BasicTypeKind::Int32) => true,
            (Value::Int64(_), BasicTypeKind::Int64) => true,
            (Value::Int128(_), BasicTypeKind::Int128) => true,
            (Value::UInt8(_), BasicTypeKind::UInt8) => true,
            (Value::UInt16(_), BasicTypeKind::UInt16) => true,
            (Value::UInt32(_), BasicTypeKind::UInt32) => true,
            (Value::UInt64(_), BasicTypeKind::UInt64) => true,
            (Value::UInt128(_), BasicTypeKind::UInt128) => true,
            (Value::Float32(_), BasicTypeKind::Float32) => true,
            (Value::Float64(_), BasicTypeKind::Float64) => true,
            (Value::String(_), BasicTypeKind::String) => true,
            (Value::Binary(_), BasicTypeKind::Binary) => true,
            (Value::Timestamp(_), BasicTypeKind::Timestamp) => true,
            (Value::Price(_), BasicTypeKind::Price) => true,
            (Value::Volume(_), BasicTypeKind::Volume) => true,
            (Value::Symbol(_), BasicTypeKind::Symbol) => true,
            (Value::ExchangeId(_), BasicTypeKind::ExchangeId) => true,
            _ => false,
        };
        
        if matches {
            Ok(())
        } else {
            Err(Error::validation(format!(
                "Value {:?} does not match type {}",
                value, info.name
            )))
        }
    }
    
    /// 验证复合类型值
    fn validate_composite_value(&self, _value: &Value, _info: &CompositeTypeInfo) -> Result<()> {
        // TODO: 实现复合类型验证
        Ok(())
    }
    
    /// 验证用户类型值
    fn validate_user_value(&self, _value: &Value, _info: &UserTypeInfo) -> Result<()> {
        // TODO: 实现用户类型验证
        Ok(())
    }
    
    /// 验证WASM类型值
    fn validate_wasm_value(&self, _value: &Value, _info: &WasmTypeInfo) -> Result<()> {
        // TODO: 实现WASM类型验证
        Ok(())
    }
    
    /// 列出所有类型
    pub fn list_types(&self) -> Vec<String> {
        self.type_name_to_id.read().keys().cloned().collect()
    }
    
    /// 获取类型统计信息
    pub fn get_stats(&self) -> TypeRegistryStats {
        TypeRegistryStats {
            basic_types_count: self.basic_types.read().len(),
            composite_types_count: self.composite_types.read().len(),
            user_types_count: self.user_types.read().len(),
            wasm_types_count: self.wasm_types.read().len(),
            total_types_count: self.type_name_to_id.read().len(),
        }
    }
}

/// 基础类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicTypeInfo {
    pub type_id: TypeId,
    pub name: String,
    pub kind: BasicTypeKind,
    pub size_bytes: usize,
    pub alignment: usize,
}

/// 基础类型种类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BasicTypeKind {
    Null,
    Bool,
    Int8, Int16, Int32, Int64, Int128,
    UInt8, UInt16, UInt32, UInt64, UInt128,
    Float32, Float64,
    String, Binary,
    Timestamp,
    Price, Volume, Symbol, ExchangeId,
}

/// 复合类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeTypeInfo {
    pub type_id: TypeId,
    pub name: String,
    pub kind: CompositeTypeKind,
    pub fields: Vec<FieldInfo>,
}

/// 复合类型种类
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositeTypeKind {
    Struct,
    Array,
    List,
    Map,
    Union,
    Tuple,
}

/// 字段信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub type_id: TypeId,
    pub optional: bool,
    pub default_value: Option<Value>,
}

/// 用户定义类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTypeInfo {
    pub type_id: TypeId,
    pub name: String,
    pub schema: TypeSchema,
    pub wasm_module: Option<String>,
    pub serializer: Option<String>,
    pub deserializer: Option<String>,
    pub validator: Option<String>,
}

/// WASM类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmTypeInfo {
    pub type_id: TypeId,
    pub name: String,
    pub wasm_module: String,
    pub type_definition: String,
    pub serializer_function: String,
    pub deserializer_function: String,
    pub validator_function: Option<String>,
}

/// 类型模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSchema {
    pub version: String,
    pub description: Option<String>,
    pub properties: HashMap<String, PropertySchema>,
    pub required: Vec<String>,
}

/// 属性模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    pub type_name: String,
    pub description: Option<String>,
    pub constraints: Option<PropertyConstraints>,
}

/// 属性约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyConstraints {
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub pattern: Option<String>,
    pub enum_values: Option<Vec<Value>>,
}

/// 类型信息枚举
#[derive(Debug, Clone)]
pub enum TypeInfo {
    Basic(BasicTypeInfo),
    Composite(CompositeTypeInfo),
    User(UserTypeInfo),
    Wasm(WasmTypeInfo),
}

/// 类型注册表统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRegistryStats {
    pub basic_types_count: usize,
    pub composite_types_count: usize,
    pub user_types_count: usize,
    pub wasm_types_count: usize,
    pub total_types_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_registry_creation() {
        let registry = TypeRegistry::new();
        let stats = registry.get_stats();
        
        // 应该有预注册的基础类型
        assert!(stats.basic_types_count > 0);
        assert!(stats.total_types_count > 0);
    }

    #[test]
    fn test_get_type_id() {
        let registry = TypeRegistry::new();
        
        let bool_type_id = registry.get_type_id("bool");
        assert!(bool_type_id.is_some());
        
        let nonexistent_type_id = registry.get_type_id("nonexistent");
        assert!(nonexistent_type_id.is_none());
    }

    #[test]
    fn test_validate_basic_value() {
        let registry = TypeRegistry::new();
        
        let bool_type_id = registry.get_type_id("bool").unwrap();
        let result = registry.validate_value(&Value::Bool(true), bool_type_id);
        assert!(result.is_ok());
        
        let result = registry.validate_value(&Value::Int32(42), bool_type_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_types() {
        let registry = TypeRegistry::new();
        let types = registry.list_types();
        
        assert!(types.contains(&"bool".to_string()));
        assert!(types.contains(&"i32".to_string()));
        assert!(types.contains(&"string".to_string()));
        assert!(types.contains(&"price".to_string()));
        assert!(types.contains(&"symbol".to_string()));
    }
}
