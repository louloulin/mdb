//! Custom Type System demonstration example

use fdc_types::{
    registry::{TypeRegistry, TypeRegistryConfig},
    definition::{TypeDefinition, TypeKind, PrimitiveType, FieldDefinition, TypeConstraint},
    validation::TypeValidator,
    conversion::TypeConverter,
    schema::{SchemaBuilder, SchemaValidation},
    financial::{create_common_financial_types, PriceType, VolumeType, CurrencyType, OptionContractType, OptionType},
    wasm_types::WasmTypeConverter,
    serialization::{TypeSerializer, SerializationFormat},
    introspection::TypeIntrospector,
};
use fdc_core::types::Value;
use rust_decimal::Decimal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Custom Type System Demo");
    println!("===========================");
    
    // 1. 演示类型注册表
    demo_type_registry()?;
    
    // 2. 演示类型定义
    demo_type_definition()?;
    
    // 3. 演示金融类型
    demo_financial_types()?;
    
    // 4. 演示类型验证
    demo_type_validation()?;
    
    // 5. 演示类型转换
    demo_type_conversion()?;
    
    // 6. 演示类型模式
    demo_type_schema()?;
    
    // 7. 演示WASM集成
    demo_wasm_integration()?;
    
    // 8. 演示序列化
    demo_serialization()?;
    
    // 9. 演示类型内省
    demo_introspection()?;
    
    println!("\n✅ All custom type system demos completed successfully!");
    Ok(())
}

fn demo_type_registry() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📚 Type Registry Demo");
    println!("--------------------");
    
    let config = TypeRegistryConfig {
        max_types: 100,
        max_fields: 50,
        max_nesting_depth: 5,
        enable_cache: true,
        enable_validation: true,
        enable_versioning: true,
    };
    
    let registry = TypeRegistry::new(config);
    
    println!("📋 Registry configuration:");
    println!("  Max types: {}", registry.config().max_types);
    println!("  Max fields: {}", registry.config().max_fields);
    println!("  Cache enabled: {}", registry.config().enable_cache);
    println!("  Validation enabled: {}", registry.config().enable_validation);
    
    // 检查内置类型
    println!("\n📊 Built-in types:");
    println!("  Has 'bool': {}", registry.has_type("bool"));
    println!("  Has 'i32': {}", registry.has_type("i32"));
    println!("  Has 'string': {}", registry.has_type("string"));
    println!("  Has 'decimal': {}", registry.has_type("decimal"));
    
    let stats = registry.get_stats();
    println!("\n📈 Registry statistics:");
    println!("  Total types: {}", stats.total_types);
    println!("  Primitive types: {}", stats.primitive_types);
    println!("  Struct types: {}", stats.struct_types);
    println!("  Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
    
    // 注册自定义类型
    let custom_type = TypeDefinition::new(
        "CustomNumber".to_string(),
        TypeKind::Primitive(PrimitiveType::F64),
    ).with_description("A custom number type".to_string());
    
    let type_id = registry.register_type(custom_type)?;
    println!("\n✅ Registered custom type with ID: {}", type_id);
    
    // 搜索类型
    let search_results = registry.search_types("number");
    println!("🔍 Search results for 'number': {} types found", search_results.len());
    
    Ok(())
}

fn demo_type_definition() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏗️ Type Definition Demo");
    println!("-----------------------");
    
    // 创建一个复杂的结构体类型
    let mut person_type = TypeDefinition::new(
        "Person".to_string(),
        TypeKind::Struct,
    ).with_description("A person data structure".to_string())
     .with_version("1.0.0".to_string());
    
    // 添加字段
    let name_field = FieldDefinition::new(
        "name".to_string(),
        TypeDefinition::new("string".to_string(), TypeKind::Primitive(PrimitiveType::String)),
    ).with_description("Person's name".to_string());
    
    let age_field = FieldDefinition::new(
        "age".to_string(),
        TypeDefinition::new("u32".to_string(), TypeKind::Primitive(PrimitiveType::U32)),
    ).optional()
     .with_description("Person's age".to_string());
    
    let email_field = FieldDefinition::new(
        "email".to_string(),
        TypeDefinition::new("string".to_string(), TypeKind::Primitive(PrimitiveType::String)),
    ).optional()
     .with_description("Person's email address".to_string());
    
    person_type.add_field(name_field);
    person_type.add_field(age_field);
    person_type.add_field(email_field);
    
    // 添加约束
    person_type.add_constraint(TypeConstraint::Pattern(r"^[A-Za-z\s]+$".to_string()));
    
    // 设置属性
    person_type.set_attribute("category".to_string(), "personal_data".to_string());
    person_type.set_attribute("version".to_string(), "1.0.0".to_string());
    
    println!("📋 Person type definition:");
    println!("  Name: {}", person_type.name);
    println!("  Description: {:?}", person_type.description);
    println!("  Version: {}", person_type.version);
    println!("  Fields: {}", person_type.fields.len());
    println!("  Constraints: {}", person_type.constraints.len());
    println!("  Is primitive: {}", person_type.is_primitive());
    println!("  Is composite: {}", person_type.is_composite());
    println!("  Size hint: {:?}", person_type.size_hint());
    
    // 验证类型定义
    person_type.validate()?;
    println!("✅ Type definition validation passed");
    
    Ok(())
}

fn demo_financial_types() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💰 Financial Types Demo");
    println!("----------------------");
    
    // 创建USD价格类型
    let usd_price = PriceType::new(
        2, // 2位小数
        Decimal::new(1, 2), // 0.01最小变动
        "USD".to_string(),
    ).with_range(
        Decimal::new(1, 4), // 最小0.0001
        Decimal::new(1000000, 0), // 最大1,000,000
    );
    
    let price_type_def = usd_price.to_type_definition("USDPrice".to_string());
    println!("📊 USD Price type:");
    println!("  Name: {}", price_type_def.name);
    println!("  Precision: {:?}", price_type_def.attributes.get("precision"));
    println!("  Currency: {:?}", price_type_def.attributes.get("currency"));
    println!("  Constraints: {}", price_type_def.constraints.len());
    
    // 创建股票成交量类型
    let stock_volume = VolumeType::new(
        1, // 最小1股
        "shares".to_string(),
    ).with_max_volume(1_000_000_000);
    
    let volume_type_def = stock_volume.to_type_definition("StockVolume".to_string());
    println!("\n📈 Stock Volume type:");
    println!("  Name: {}", volume_type_def.name);
    println!("  Lot size: {:?}", volume_type_def.attributes.get("lot_size"));
    println!("  Unit: {:?}", volume_type_def.attributes.get("unit"));
    
    // 创建期权合约类型
    let option_contract = OptionContractType::new(
        "AAPL".to_string(),
        OptionType::Call,
        usd_price,
        100, // 标准合约乘数
    );
    
    let option_type_def = option_contract.to_type_definition("AAPLCallOption".to_string());
    println!("\n📋 AAPL Call Option type:");
    println!("  Name: {}", option_type_def.name);
    println!("  Fields: {}", option_type_def.fields.len());
    println!("  Underlying: {:?}", option_type_def.attributes.get("underlying"));
    println!("  Option type: {:?}", option_type_def.attributes.get("option_type"));
    
    // 创建常用金融类型
    let common_types = create_common_financial_types();
    println!("\n📚 Common financial types created: {}", common_types.len());
    for type_def in &common_types {
        println!("  - {}: {:?}", type_def.name, type_def.description);
    }
    
    Ok(())
}

fn demo_type_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n✅ Type Validation Demo");
    println!("----------------------");
    
    let validator = TypeValidator::new();
    
    // 创建一个带约束的类型
    let mut price_type = TypeDefinition::new(
        "Price".to_string(),
        TypeKind::Primitive(PrimitiveType::F64),
    );
    price_type.add_constraint(TypeConstraint::MinValue(0.0));
    price_type.add_constraint(TypeConstraint::MaxValue(1000000.0));
    
    // 验证有效值
    let valid_price = Value::Float64(150.25);
    let errors = validator.validate_value(&valid_price, &price_type)?;
    println!("📊 Validation of valid price (150.25):");
    println!("  Errors: {}", errors.len());
    
    // 验证无效值
    let invalid_price = Value::Float64(-10.0);
    let errors = validator.validate_value(&invalid_price, &price_type)?;
    println!("\n📊 Validation of invalid price (-10.0):");
    println!("  Errors: {}", errors.len());
    if !errors.is_empty() {
        println!("  Error: {}", errors[0].message);
    }
    
    Ok(())
}

fn demo_type_conversion() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Type Conversion Demo");
    println!("----------------------");
    
    let converter = TypeConverter::new();
    
    // 转换示例
    let int_value = Value::Int32(42);
    println!("📊 Converting i32(42) to i64:");
    match converter.convert(&int_value, "i64") {
        Ok(converted) => println!("  Result: {:?}", converted),
        Err(e) => println!("  Error: {}", e),
    }
    
    let float_value = Value::Float32(3.14);
    println!("\n📊 Converting f32(3.14) to f64:");
    match converter.convert(&float_value, "f64") {
        Ok(converted) => println!("  Result: {:?}", converted),
        Err(e) => println!("  Error: {}", e),
    }
    
    Ok(())
}

fn demo_type_schema() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Type Schema Demo");
    println!("------------------");
    
    // 创建类型模式
    let person_type = TypeDefinition::new(
        "Person".to_string(),
        TypeKind::Struct,
    );
    
    let address_type = TypeDefinition::new(
        "Address".to_string(),
        TypeKind::Struct,
    );
    
    let schema = SchemaBuilder::new("PersonSchema".to_string())
        .version("1.0.0".to_string())
        .add_type(person_type)
        .add_type(address_type)
        .build();
    
    println!("📊 Schema information:");
    println!("  Name: {}", schema.name);
    println!("  Version: {}", schema.version);
    println!("  Types: {}", schema.types.len());
    
    // 验证模式
    SchemaValidation::validate_schema(&schema)?;
    println!("✅ Schema validation passed");
    
    Ok(())
}

fn demo_wasm_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌉 WASM Integration Demo");
    println!("-----------------------");
    
    let wasm_converter = WasmTypeConverter::new();
    
    // 转换到WASM值
    let core_value = Value::Float64(123.45);
    let wasm_value = wasm_converter.to_wasm_value(&core_value)?;
    println!("📊 Core to WASM conversion:");
    println!("  Core value: {:?}", core_value);
    println!("  WASM value: {:?}", wasm_value);
    
    // 转换回核心值
    let converted_back = wasm_converter.from_wasm_value(&wasm_value)?;
    println!("\n📊 WASM to Core conversion:");
    println!("  WASM value: {:?}", wasm_value);
    println!("  Core value: {:?}", converted_back);
    
    println!("✅ Round-trip conversion successful: {}", core_value == converted_back);
    
    Ok(())
}

fn demo_serialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 Serialization Demo");
    println!("--------------------");
    
    let serializer = TypeSerializer::new();
    
    let type_def = TypeDefinition::new(
        "TestType".to_string(),
        TypeKind::Primitive(PrimitiveType::String),
    ).with_description("A test type for serialization".to_string());
    
    // JSON序列化
    let json_data = serializer.serialize(&type_def, SerializationFormat::Json)?;
    println!("📊 JSON serialization:");
    println!("  Size: {} bytes", json_data.len());
    
    let deserialized_json = serializer.deserialize(&json_data, SerializationFormat::Json)?;
    println!("  Deserialized name: {}", deserialized_json.name);
    
    // 二进制序列化
    let binary_data = serializer.serialize(&type_def, SerializationFormat::Binary)?;
    println!("\n📊 Binary serialization:");
    println!("  Size: {} bytes", binary_data.len());
    
    let deserialized_binary = serializer.deserialize(&binary_data, SerializationFormat::Binary)?;
    println!("  Deserialized name: {}", deserialized_binary.name);
    
    println!("✅ Serialization round-trip successful");
    
    Ok(())
}

fn demo_introspection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Type Introspection Demo");
    println!("-------------------------");
    
    let introspector = TypeIntrospector::new();
    
    // 内省基础类型
    let int_type = TypeDefinition::new(
        "i64".to_string(),
        TypeKind::Primitive(PrimitiveType::I64),
    );
    
    let metadata = introspector.introspect(&int_type);
    println!("📊 i64 type metadata:");
    println!("  Name: {}", metadata.name);
    println!("  Size hint: {:?}", metadata.size_hint);
    println!("  Is primitive: {}", metadata.is_primitive);
    println!("  Is composite: {}", metadata.is_composite);
    println!("  Field count: {}", metadata.field_count);
    
    // 内省复杂类型
    let mut struct_type = TypeDefinition::new(
        "ComplexStruct".to_string(),
        TypeKind::Struct,
    );
    struct_type.set_attribute("category".to_string(), "complex".to_string());
    struct_type.add_field(FieldDefinition::new(
        "field1".to_string(),
        TypeDefinition::new("string".to_string(), TypeKind::Primitive(PrimitiveType::String)),
    ));
    
    let complex_metadata = introspector.introspect(&struct_type);
    println!("\n📊 ComplexStruct type metadata:");
    println!("  Name: {}", complex_metadata.name);
    println!("  Is primitive: {}", complex_metadata.is_primitive);
    println!("  Is composite: {}", complex_metadata.is_composite);
    println!("  Field count: {}", complex_metadata.field_count);
    println!("  Attributes: {}", complex_metadata.attributes.len());
    
    Ok(())
}
