//! Type schema system

use crate::definition::TypeDefinition;
use fdc_core::error::Result;
use serde::{Deserialize, Serialize};

/// 类型模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSchema {
    pub name: String,
    pub version: String,
    pub types: Vec<TypeDefinition>,
}

/// 模式构建器
pub struct SchemaBuilder {
    name: String,
    version: String,
    types: Vec<TypeDefinition>,
}

impl SchemaBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: "1.0.0".to_string(),
            types: Vec::new(),
        }
    }
    
    pub fn version(mut self, version: String) -> Self {
        self.version = version;
        self
    }
    
    pub fn add_type(mut self, type_def: TypeDefinition) -> Self {
        self.types.push(type_def);
        self
    }
    
    pub fn build(self) -> TypeSchema {
        TypeSchema {
            name: self.name,
            version: self.version,
            types: self.types,
        }
    }
}

/// 模式验证
pub struct SchemaValidation;

impl SchemaValidation {
    pub fn validate_schema(schema: &TypeSchema) -> Result<()> {
        // 简化的模式验证
        for type_def in &schema.types {
            type_def.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{TypeDefinition, TypeKind, PrimitiveType};

    #[test]
    fn test_schema_builder() {
        let type_def = TypeDefinition::new("test".to_string(), TypeKind::Primitive(PrimitiveType::I32));
        let schema = SchemaBuilder::new("TestSchema".to_string())
            .version("1.0.0".to_string())
            .add_type(type_def)
            .build();
        
        assert_eq!(schema.name, "TestSchema");
        assert_eq!(schema.types.len(), 1);
    }
}
