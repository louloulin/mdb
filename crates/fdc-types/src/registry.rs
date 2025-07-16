//! Type registry implementation

use crate::definition::{TypeDefinition, TypeKind, PrimitiveType};
use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

/// 类型注册表配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRegistryConfig {
    /// 最大类型数量
    pub max_types: usize,
    /// 最大字段数量
    pub max_fields: usize,
    /// 最大嵌套深度
    pub max_nesting_depth: usize,
    /// 是否启用类型缓存
    pub enable_cache: bool,
    /// 是否启用类型验证
    pub enable_validation: bool,
    /// 是否启用版本控制
    pub enable_versioning: bool,
}

impl Default for TypeRegistryConfig {
    fn default() -> Self {
        Self {
            max_types: crate::DEFAULT_MAX_TYPES,
            max_fields: crate::DEFAULT_MAX_FIELDS,
            max_nesting_depth: crate::DEFAULT_MAX_NESTING_DEPTH,
            enable_cache: true,
            enable_validation: true,
            enable_versioning: true,
        }
    }
}

/// 类型注册表统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeRegistryStats {
    /// 注册的类型总数
    pub total_types: usize,
    /// 基础类型数量
    pub primitive_types: usize,
    /// 结构体类型数量
    pub struct_types: usize,
    /// 枚举类型数量
    pub enum_types: usize,
    /// 自定义类型数量
    pub custom_types: usize,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 验证成功次数
    pub validation_successes: u64,
    /// 验证失败次数
    pub validation_failures: u64,
}

impl TypeRegistryStats {
    /// 获取缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
    
    /// 获取验证成功率
    pub fn validation_success_rate(&self) -> f64 {
        let total = self.validation_successes + self.validation_failures;
        if total == 0 {
            0.0
        } else {
            self.validation_successes as f64 / total as f64
        }
    }
}

/// 类型注册表
pub struct TypeRegistry {
    /// 配置
    config: TypeRegistryConfig,
    /// 按ID索引的类型
    types_by_id: Arc<RwLock<HashMap<Uuid, TypeDefinition>>>,
    /// 按名称索引的类型
    types_by_name: Arc<RwLock<HashMap<String, Uuid>>>,
    /// 类型缓存
    type_cache: Arc<RwLock<HashMap<String, TypeDefinition>>>,
    /// 统计信息
    stats: Arc<RwLock<TypeRegistryStats>>,
}

impl TypeRegistry {
    /// 创建新的类型注册表
    pub fn new(config: TypeRegistryConfig) -> Self {
        let registry = Self {
            config,
            types_by_id: Arc::new(RwLock::new(HashMap::new())),
            types_by_name: Arc::new(RwLock::new(HashMap::new())),
            type_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(TypeRegistryStats::default())),
        };
        
        // 注册内置基础类型
        registry.register_builtin_types();
        
        registry
    }
    
    /// 注册内置基础类型
    fn register_builtin_types(&self) {
        let primitive_types = [
            ("bool", PrimitiveType::Bool),
            ("i8", PrimitiveType::I8),
            ("i16", PrimitiveType::I16),
            ("i32", PrimitiveType::I32),
            ("i64", PrimitiveType::I64),
            ("i128", PrimitiveType::I128),
            ("u8", PrimitiveType::U8),
            ("u16", PrimitiveType::U16),
            ("u32", PrimitiveType::U32),
            ("u64", PrimitiveType::U64),
            ("u128", PrimitiveType::U128),
            ("f32", PrimitiveType::F32),
            ("f64", PrimitiveType::F64),
            ("string", PrimitiveType::String),
            ("bytes", PrimitiveType::Bytes),
            ("timestamp", PrimitiveType::Timestamp),
            ("decimal", PrimitiveType::Decimal),
            ("bigint", PrimitiveType::BigInt),
        ];
        
        for (name, prim_type) in &primitive_types {
            let type_def = TypeDefinition::new(
                name.to_string(),
                TypeKind::Primitive(prim_type.clone()),
            );
            
            // 直接插入，不进行验证（内置类型总是有效的）
            let mut types_by_id = self.types_by_id.write();
            let mut types_by_name = self.types_by_name.write();
            
            types_by_id.insert(type_def.id, type_def.clone());
            types_by_name.insert(name.to_string(), type_def.id);
        }
        
        // 更新统计
        let mut stats = self.stats.write();
        stats.primitive_types = primitive_types.len();
        stats.total_types = primitive_types.len();
    }
    
    /// 注册类型
    pub fn register_type(&self, type_def: TypeDefinition) -> Result<Uuid> {
        // 检查是否达到最大类型数量
        {
            let types_by_id = self.types_by_id.read();
            if types_by_id.len() >= self.config.max_types {
                return Err(Error::resource_exhausted("Maximum number of types reached"));
            }
        }
        
        // 验证类型定义
        if self.config.enable_validation {
            type_def.validate()?;
            self.stats.write().validation_successes += 1;
        }
        
        // 检查名称是否已存在
        {
            let types_by_name = self.types_by_name.read();
            if types_by_name.contains_key(&type_def.name) {
                return Err(Error::already_exists(format!("Type name: {}", type_def.name)));
            }
        }
        
        let type_id = type_def.id;
        let type_name = type_def.name.clone();
        
        // 注册类型
        {
            let mut types_by_id = self.types_by_id.write();
            let mut types_by_name = self.types_by_name.write();
            
            types_by_id.insert(type_id, type_def.clone());
            types_by_name.insert(type_name, type_id);
        }
        
        // 更新统计
        {
            let mut stats = self.stats.write();
            stats.total_types += 1;
            match type_def.kind {
                TypeKind::Struct => stats.struct_types += 1,
                TypeKind::Enum => stats.enum_types += 1,
                TypeKind::Custom(_) => stats.custom_types += 1,
                _ => {}
            }
        }
        
        // 清除缓存
        if self.config.enable_cache {
            self.type_cache.write().clear();
        }
        
        Ok(type_id)
    }
    
    /// 根据ID获取类型
    pub fn get_type_by_id(&self, type_id: Uuid) -> Option<TypeDefinition> {
        let types_by_id = self.types_by_id.read();
        types_by_id.get(&type_id).cloned()
    }
    
    /// 根据名称获取类型
    pub fn get_type_by_name(&self, name: &str) -> Option<TypeDefinition> {
        // 检查缓存
        if self.config.enable_cache {
            let cache = self.type_cache.read();
            if let Some(type_def) = cache.get(name) {
                self.stats.write().cache_hits += 1;
                return Some(type_def.clone());
            }
            self.stats.write().cache_misses += 1;
        }
        
        // 从注册表获取
        let types_by_name = self.types_by_name.read();
        let type_id = types_by_name.get(name)?;
        
        let types_by_id = self.types_by_id.read();
        let type_def = types_by_id.get(type_id)?.clone();
        
        // 更新缓存
        if self.config.enable_cache {
            self.type_cache.write().insert(name.to_string(), type_def.clone());
        }
        
        Some(type_def)
    }
    
    /// 卸载类型
    pub fn unregister_type(&self, type_id: Uuid) -> Result<()> {
        let type_name = {
            let types_by_id = self.types_by_id.read();
            let type_def = types_by_id.get(&type_id)
                .ok_or_else(|| Error::not_found(format!("Type ID: {}", type_id)))?;
            type_def.name.clone()
        };
        
        // 检查是否有其他类型依赖此类型
        self.check_dependencies(&type_name)?;
        
        // 移除类型
        {
            let mut types_by_id = self.types_by_id.write();
            let mut types_by_name = self.types_by_name.write();
            
            if let Some(type_def) = types_by_id.remove(&type_id) {
                types_by_name.remove(&type_def.name);
                
                // 更新统计
                let mut stats = self.stats.write();
                stats.total_types -= 1;
                match type_def.kind {
                    TypeKind::Struct => stats.struct_types -= 1,
                    TypeKind::Enum => stats.enum_types -= 1,
                    TypeKind::Custom(_) => stats.custom_types -= 1,
                    _ => {}
                }
            }
        }
        
        // 清除缓存
        if self.config.enable_cache {
            self.type_cache.write().remove(&type_name);
        }
        
        Ok(())
    }
    
    /// 检查类型依赖
    fn check_dependencies(&self, type_name: &str) -> Result<()> {
        let types_by_id = self.types_by_id.read();
        
        for type_def in types_by_id.values() {
            if self.type_depends_on(type_def, type_name) {
                return Err(Error::validation(format!(
                    "Cannot unregister type '{}': it is used by type '{}'",
                    type_name, type_def.name
                )));
            }
        }
        
        Ok(())
    }
    
    /// 检查类型是否依赖指定类型
    fn type_depends_on(&self, type_def: &TypeDefinition, target_name: &str) -> bool {
        match &type_def.kind {
            TypeKind::Reference(name) => name == target_name,
            TypeKind::Array(inner) => self.type_depends_on(inner, target_name),
            TypeKind::Map(key, value) => {
                self.type_depends_on(key, target_name) || self.type_depends_on(value, target_name)
            }
            TypeKind::Optional(inner) => self.type_depends_on(inner, target_name),
            TypeKind::Function { inputs, output } => {
                inputs.iter().any(|input| self.type_depends_on(input, target_name))
                    || self.type_depends_on(output, target_name)
            }
            _ => {
                // 检查字段
                type_def.fields.iter().any(|field| {
                    self.type_depends_on(&field.field_type, target_name)
                })
            }
        }
    }
    
    /// 列出所有类型
    pub fn list_types(&self) -> Vec<TypeDefinition> {
        self.types_by_id.read().values().cloned().collect()
    }
    
    /// 列出类型名称
    pub fn list_type_names(&self) -> Vec<String> {
        self.types_by_name.read().keys().cloned().collect()
    }
    
    /// 按种类列出类型
    pub fn list_types_by_kind(&self, kind: &TypeKind) -> Vec<TypeDefinition> {
        self.types_by_id.read()
            .values()
            .filter(|type_def| std::mem::discriminant(&type_def.kind) == std::mem::discriminant(kind))
            .cloned()
            .collect()
    }
    
    /// 搜索类型
    pub fn search_types(&self, query: &str) -> Vec<TypeDefinition> {
        let query_lower = query.to_lowercase();
        self.types_by_id.read()
            .values()
            .filter(|type_def| {
                type_def.name.to_lowercase().contains(&query_lower)
                    || type_def.description.as_ref()
                        .map(|desc| desc.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
    
    /// 获取类型数量
    pub fn type_count(&self) -> usize {
        self.types_by_id.read().len()
    }
    
    /// 检查类型是否存在
    pub fn has_type(&self, name: &str) -> bool {
        self.types_by_name.read().contains_key(name)
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> TypeRegistryStats {
        self.stats.read().clone()
    }
    
    /// 清除缓存
    pub fn clear_cache(&self) {
        if self.config.enable_cache {
            self.type_cache.write().clear();
        }
    }
    
    /// 获取配置
    pub fn config(&self) -> &TypeRegistryConfig {
        &self.config
    }
    
    /// 验证类型兼容性
    pub fn is_compatible(&self, from_type: &str, to_type: &str) -> bool {
        // 简化的兼容性检查
        if from_type == to_type {
            return true;
        }
        
        // 数值类型之间的兼容性
        let numeric_types = [
            "i8", "i16", "i32", "i64", "i128",
            "u8", "u16", "u32", "u64", "u128",
            "f32", "f64", "decimal", "bigint"
        ];
        
        if numeric_types.contains(&from_type) && numeric_types.contains(&to_type) {
            return true;
        }
        
        false
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new(TypeRegistryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{TypeDefinition, TypeKind, PrimitiveType};

    #[test]
    fn test_registry_creation() {
        let registry = TypeRegistry::default();
        let stats = registry.get_stats();
        
        // 应该有内置的基础类型
        assert!(stats.total_types > 0);
        assert!(stats.primitive_types > 0);
        assert_eq!(stats.struct_types, 0);
        assert_eq!(stats.enum_types, 0);
    }

    #[test]
    fn test_type_registration() {
        let registry = TypeRegistry::default();
        
        let type_def = TypeDefinition::new(
            "CustomType".to_string(),
            TypeKind::Struct,
        );
        
        let type_id = registry.register_type(type_def.clone()).unwrap();
        assert_eq!(type_id, type_def.id);
        
        // 检查类型是否存在
        assert!(registry.has_type("CustomType"));
        
        // 获取类型
        let retrieved = registry.get_type_by_name("CustomType").unwrap();
        assert_eq!(retrieved.name, "CustomType");
        
        let retrieved_by_id = registry.get_type_by_id(type_id).unwrap();
        assert_eq!(retrieved_by_id.name, "CustomType");
    }

    #[test]
    fn test_builtin_types() {
        let registry = TypeRegistry::default();
        
        assert!(registry.has_type("bool"));
        assert!(registry.has_type("i32"));
        assert!(registry.has_type("string"));
        assert!(registry.has_type("timestamp"));
        
        let bool_type = registry.get_type_by_name("bool").unwrap();
        assert_eq!(bool_type.name, "bool");
        assert!(bool_type.is_primitive());
    }

    #[test]
    fn test_type_search() {
        let registry = TypeRegistry::default();
        
        let results = registry.search_types("int");
        assert!(!results.is_empty());
        
        // 应该找到包含"int"的类型
        let has_bigint = results.iter().any(|t| t.name == "bigint");
        assert!(has_bigint);
    }

    #[test]
    fn test_type_compatibility() {
        let registry = TypeRegistry::default();
        
        // 相同类型应该兼容
        assert!(registry.is_compatible("i32", "i32"));
        
        // 数值类型之间应该兼容
        assert!(registry.is_compatible("i32", "i64"));
        assert!(registry.is_compatible("f32", "f64"));
        
        // 不同类别的类型不兼容
        assert!(!registry.is_compatible("string", "i32"));
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = TypeRegistry::default();
        
        let type_def1 = TypeDefinition::new(
            "DuplicateType".to_string(),
            TypeKind::Struct,
        );
        
        let type_def2 = TypeDefinition::new(
            "DuplicateType".to_string(),
            TypeKind::Enum,
        );
        
        // 第一次注册应该成功
        assert!(registry.register_type(type_def1).is_ok());
        
        // 第二次注册相同名称应该失败
        assert!(registry.register_type(type_def2).is_err());
    }
}
