//! Type serialization support

use crate::definition::TypeDefinition;
use fdc_core::error::Result;
use serde::{Deserialize, Serialize};

/// 序列化格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    Json,
    Binary,
    MessagePack,
    Protobuf,
}

/// 类型序列化器
pub struct TypeSerializer;

impl TypeSerializer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn serialize(&self, type_def: &TypeDefinition, format: SerializationFormat) -> Result<Vec<u8>> {
        match format {
            SerializationFormat::Json => {
                let json = serde_json::to_vec(type_def)?;
                Ok(json)
            }
            SerializationFormat::Binary => {
                let binary = bincode::serialize(type_def)?;
                Ok(binary)
            }
            _ => Ok(Vec::new()), // 简化实现
        }
    }
    
    pub fn deserialize(&self, data: &[u8], format: SerializationFormat) -> Result<TypeDefinition> {
        match format {
            SerializationFormat::Json => {
                let type_def = serde_json::from_slice(data)?;
                Ok(type_def)
            }
            SerializationFormat::Binary => {
                let type_def = bincode::deserialize(data)?;
                Ok(type_def)
            }
            _ => Err(fdc_core::error::Error::unimplemented("Format not supported")),
        }
    }
}

impl Default for TypeSerializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{TypeDefinition, TypeKind, PrimitiveType};

    #[test]
    fn test_serialization() {
        let serializer = TypeSerializer::new();
        let type_def = TypeDefinition::new("test".to_string(), TypeKind::Primitive(PrimitiveType::I32));
        
        let data = serializer.serialize(&type_def, SerializationFormat::Json).unwrap();
        let deserialized = serializer.deserialize(&data, SerializationFormat::Json).unwrap();
        
        assert_eq!(type_def.name, deserialized.name);
    }
}
