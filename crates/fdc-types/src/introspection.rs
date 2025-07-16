//! Type introspection system

use crate::definition::TypeDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 类型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMetadata {
    pub name: String,
    pub size_hint: Option<usize>,
    pub is_primitive: bool,
    pub is_composite: bool,
    pub field_count: usize,
    pub constraint_count: usize,
    pub attributes: HashMap<String, String>,
}

/// 类型内省器
pub struct TypeIntrospector;

impl TypeIntrospector {
    pub fn new() -> Self {
        Self
    }
    
    pub fn introspect(&self, type_def: &TypeDefinition) -> TypeMetadata {
        TypeMetadata {
            name: type_def.name.clone(),
            size_hint: type_def.size_hint(),
            is_primitive: type_def.is_primitive(),
            is_composite: type_def.is_composite(),
            field_count: type_def.fields.len(),
            constraint_count: type_def.constraints.len(),
            attributes: type_def.attributes.clone(),
        }
    }
}

impl Default for TypeIntrospector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{TypeDefinition, TypeKind, PrimitiveType};

    #[test]
    fn test_introspection() {
        let introspector = TypeIntrospector::new();
        let type_def = TypeDefinition::new("test".to_string(), TypeKind::Primitive(PrimitiveType::I32));
        
        let metadata = introspector.introspect(&type_def);
        assert_eq!(metadata.name, "test");
        assert!(metadata.is_primitive);
        assert!(!metadata.is_composite);
    }
}
