//! # Financial Data Center Custom Type System
//!
//! This crate provides a comprehensive custom type system for the
//! Financial Data Center, enabling user-defined types, WASM-driven
//! type conversions, and dynamic type validation.

pub mod registry;       // 类型注册表
pub mod definition;     // 类型定义
pub mod validation;     // 类型验证
pub mod conversion;     // 类型转换
pub mod schema;         // 类型模式
pub mod financial;      // 金融专用类型
pub mod wasm_types;     // WASM类型集成
pub mod serialization;  // 序列化支持
pub mod introspection;  // 类型内省

// 重新导出常用类型
pub use registry::{TypeRegistry, TypeRegistryConfig};
pub use definition::{TypeDefinition, TypeKind, FieldDefinition};
pub use validation::{TypeValidator, ValidationRule, ValidationError};
pub use conversion::{TypeConverter, ConversionRule, ConversionError};
pub use schema::{TypeSchema, SchemaBuilder, SchemaValidation};
pub use financial::{
    FinancialType, PriceType, VolumeType, CurrencyType,
    OptionContractType, FutureContractType
};
pub use wasm_types::{WasmTypeIntegration, WasmTypeConverter};
pub use serialization::{TypeSerializer, SerializationFormat};
pub use introspection::{TypeIntrospector, TypeMetadata};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 默认最大类型数量
pub const DEFAULT_MAX_TYPES: usize = 1000;

/// 默认最大字段数量
pub const DEFAULT_MAX_FIELDS: usize = 100;

/// 默认最大嵌套深度
pub const DEFAULT_MAX_NESTING_DEPTH: usize = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "fdc-types");
        assert_eq!(DEFAULT_MAX_TYPES, 1000);
        assert_eq!(DEFAULT_MAX_FIELDS, 100);
        assert_eq!(DEFAULT_MAX_NESTING_DEPTH, 10);
    }
}
