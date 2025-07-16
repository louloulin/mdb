//! FDC Core demonstration example

use fdc_core::{
    types::*,
    config::Config,
    metrics::Metrics,
    time::{TimeUtils, TimeRange, intervals},
    memory::{MemoryPool, ZeroCopyBuffer, MemoryMonitor},
    type_registry::TypeRegistry,
    error::Result,
};

fn main() -> Result<()> {
    println!("🚀 Financial Data Center Core Demo");
    println!("==================================");
    
    // 1. 演示核心数据类型
    demo_core_types()?;
    
    // 2. 演示配置管理
    demo_config()?;
    
    // 3. 演示指标收集
    demo_metrics()?;
    
    // 4. 演示时间处理
    demo_time_utils()?;
    
    // 5. 演示内存管理
    demo_memory_management()?;
    
    // 6. 演示类型注册表
    demo_type_registry()?;
    
    println!("\n✅ All demos completed successfully!");
    Ok(())
}

fn demo_core_types() -> Result<()> {
    println!("\n📊 Core Data Types Demo");
    println!("-----------------------");
    
    // 创建一个tick数据
    let symbol = Symbol::new("AAPL");
    let price = Price::from_f64(150.25).unwrap();
    let volume = Volume::new(1000);
    let exchange_id = ExchangeId::new(1);
    let sequence_number = SequenceNumber::new(12345);
    
    let mut tick_data = TickData::new(
        symbol.clone(),
        price,
        volume,
        exchange_id,
        MessageType::Trade,
        sequence_number,
    );
    
    // 添加自定义字段
    tick_data.custom_fields.insert(
        "trader_id".to_string(),
        Value::String("TRADER_001".to_string()),
    );
    
    println!("📈 Created tick data:");
    println!("  Symbol: {}", tick_data.symbol);
    println!("  Price: ${}", tick_data.price);
    println!("  Volume: {}", tick_data.volume);
    println!("  Timestamp: {}", tick_data.timestamp);
    println!("  Custom fields: {:?}", tick_data.custom_fields.get("trader_id"));
    
    Ok(())
}

fn demo_config() -> Result<()> {
    println!("\n⚙️  Configuration Demo");
    println!("---------------------");
    
    let config = Config::default();
    println!("📋 Default configuration:");
    println!("  Server host: {}", config.server.host);
    println!("  REST port: {}", config.server.rest_port);
    println!("  gRPC port: {}", config.server.grpc_port);
    println!("  Realtime storage engine: {}", config.storage.realtime.engine);
    println!("  Analytical storage engine: {}", config.storage.analytical.engine);
    
    // 验证配置
    config.validate()?;
    println!("✅ Configuration validation passed");
    
    Ok(())
}

fn demo_metrics() -> Result<()> {
    println!("\n📊 Metrics Demo");
    println!("---------------");
    
    let metrics = Metrics::new();
    
    // 记录一些指标
    metrics.increment_counter("messages_processed", 100);
    metrics.increment_counter("messages_processed", 50);
    metrics.set_gauge("active_connections", 25);
    metrics.record_histogram("processing_time", 1.5);
    metrics.record_histogram("processing_time", 2.3);
    metrics.record_histogram("processing_time", 0.8);
    
    println!("📈 Recorded metrics:");
    println!("  Messages processed: {}", metrics.get_counter("messages_processed").unwrap_or(0));
    println!("  Active connections: {}", metrics.get_gauge("active_connections").unwrap_or(0));
    
    if let Some(stats) = metrics.get_histogram_stats("processing_time") {
        println!("  Processing time stats:");
        println!("    Count: {}", stats.count);
        println!("    Mean: {:.2}ms", stats.mean);
        println!("    P95: {:.2}ms", stats.p95);
        println!("    P99: {:.2}ms", stats.p99);
    }
    
    Ok(())
}

fn demo_time_utils() -> Result<()> {
    println!("\n⏰ Time Utilities Demo");
    println!("---------------------");
    
    let now = TimestampNs::now();
    println!("🕐 Current timestamp: {}", now);
    
    // 解析时间戳
    let parsed = TimeUtils::parse_timestamp("2023-01-01 12:00:00")?;
    println!("📅 Parsed timestamp: {}", parsed);
    
    // 时间范围
    let start = TimestampNs::from_nanos(1000000000);
    let end = TimestampNs::from_nanos(2000000000);
    let range = TimeRange::new(start, end)?;
    
    println!("📊 Time range:");
    println!("  Start: {}", range.start);
    println!("  End: {}", range.end);
    println!("  Duration: {}ns", range.duration_nanos());
    
    // 时间间隔
    let one_second_ago = TimeUtils::sub_nanos(now, intervals::SECOND);
    println!("⏪ One second ago: {}", one_second_ago);
    
    Ok(())
}

fn demo_memory_management() -> Result<()> {
    println!("\n🧠 Memory Management Demo");
    println!("------------------------");
    
    // 内存池
    let mut pool = MemoryPool::new(2, 1024);
    let value = pool.alloc(42u32)?;
    println!("💾 Allocated value from pool: {}", value);
    
    let usage = pool.current_usage();
    println!("📊 Pool usage: {:.2}%", usage.utilization * 100.0);
    
    // 零拷贝缓冲区
    let mut buffer = ZeroCopyBuffer::new(1024);
    let data = b"Hello, Financial Data Center!";
    buffer.write(data)?;
    
    let read_data = buffer.read(0, data.len())?;
    println!("📝 Buffer content: {}", String::from_utf8_lossy(read_data));
    println!("📏 Buffer usage: {}/{} bytes", buffer.len(), buffer.capacity());
    
    // 内存监控
    let monitor = MemoryMonitor::new();
    monitor.record_allocation(1024);
    monitor.record_allocation(512);
    monitor.record_deallocation(256);
    
    let stats = monitor.get_stats();
    println!("📈 Memory stats:");
    println!("  Total allocated: {} bytes", stats.total_allocated);
    println!("  Current usage: {} bytes", stats.current_usage);
    println!("  Peak usage: {} bytes", stats.peak_usage);
    
    Ok(())
}

fn demo_type_registry() -> Result<()> {
    println!("\n🏷️  Type Registry Demo");
    println!("---------------------");
    
    let registry = TypeRegistry::new();
    
    // 获取内置类型
    let bool_type_id = registry.get_type_id("bool").unwrap();
    let price_type_id = registry.get_type_id("price").unwrap();
    
    println!("🔍 Found type IDs:");
    println!("  bool: {:?}", bool_type_id);
    println!("  price: {:?}", price_type_id);
    
    // 验证值
    let bool_value = Value::Bool(true);
    let price_value = Value::Price(Price::from_f64(100.0).unwrap());
    
    registry.validate_value(&bool_value, bool_type_id)?;
    registry.validate_value(&price_value, price_type_id)?;
    println!("✅ Value validation passed");
    
    // 列出所有类型
    let types = registry.list_types();
    println!("📋 Available types: {}", types.join(", "));
    
    let stats = registry.get_stats();
    println!("📊 Registry stats:");
    println!("  Basic types: {}", stats.basic_types_count);
    println!("  Total types: {}", stats.total_types_count);
    
    Ok(())
}
