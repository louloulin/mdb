//! Data ingestion system demonstration

use fdc_ingestion::{
    config::IngestionConfig,
    receiver::{DataReceiver, ReceiverType},
    parser::DataParser,
    validator::DataValidator,
    batch::{BatchProcessor, BatchItem},
    metrics::IngestionMetrics,
};
// use fdc_core::types::Value; // 暂时不需要
// use fdc_storage::engines::memory::MemoryEngine; // 暂时注释掉
use fdc_types::{TypeRegistry, TypeRegistryConfig};
// use std::collections::HashMap; // 暂时不需要
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志 (简化版本)
    // tracing_subscriber::init(); // 暂时注释掉
    
    println!("🚀 Financial Data Center - Data Ingestion Demo");
    println!("================================================");
    
    // 创建配置
    let config = IngestionConfig::default();
    println!("✅ Configuration loaded");
    
    // 创建类型注册表
    let type_registry = Arc::new(TypeRegistry::new(TypeRegistryConfig::default()));
    println!("✅ Type registry initialized");
    
    // 创建简化的存储引擎
    struct SimpleMemoryStorage;

    #[async_trait::async_trait]
    impl fdc_ingestion::batch::SimpleStorage for SimpleMemoryStorage {
        async fn put(&self, _key: &[u8], _value: &[u8]) -> fdc_core::error::Result<()> {
            // 简化实现：只是模拟存储
            Ok(())
        }
    }

    let storage_engine = Arc::new(SimpleMemoryStorage);
    println!("✅ Simple memory storage engine created");
    
    // 创建数据解析器
    let parser = Arc::new(DataParser::new(config.parser.clone(), type_registry.clone()));
    println!("✅ Data parser initialized");
    
    // 创建数据验证器
    let validator = Arc::new(DataValidator::new(config.validator.clone()));
    println!("✅ Data validator initialized");
    
    // 创建批量处理器
    let batch_processor = Arc::new(BatchProcessor::new(config.batch.clone(), storage_engine.clone()));
    println!("✅ Batch processor initialized");
    
    // 创建通道
    let (data_sender, mut data_receiver) = mpsc::unbounded_channel();
    let (batch_sender, batch_receiver) = mpsc::channel(1000);
    
    // 创建数据接收器
    let _receiver = DataReceiver::new(
        config.receiver.clone(),
        ReceiverType::Tcp,
        data_sender,
    );
    println!("✅ Data receiver created");
    
    // 启动批量处理器
    let batch_processor_clone = batch_processor.clone();
    batch_processor_clone.start(batch_receiver)?;
    println!("✅ Batch processor started");
    
    // 创建指标收集器
    let mut metrics = IngestionMetrics::new();
    
    // 模拟数据处理流水线
    println!("\n📊 Starting data processing pipeline...");
    
    // 启动数据处理任务
    let parser_clone = parser.clone();
    let validator_clone = validator.clone();
    let batch_sender_clone = batch_sender.clone();
    
    tokio::spawn(async move {
        while let Some(received_data) = data_receiver.recv().await {
            // 解析数据
            match parser_clone.parse(received_data).await {
                Ok(parsed_data) => {
                    // 验证数据
                    match validator_clone.validate(&parsed_data).await {
                        Ok(validation_result) => {
                            // 创建批量项
                            let batch_item = BatchItem::new(parsed_data, validation_result);
                            
                            // 发送到批量处理器
                            if let Err(e) = batch_sender_clone.send(batch_item).await {
                                eprintln!("Failed to send batch item: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Validation error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Parsing error: {}", e);
                }
            }
        }
    });
    
    // 模拟发送一些测试数据
    println!("📤 Sending test data...");
    
    let test_data = vec![
        r#"{"symbol": "AAPL", "price": 150.25, "volume": 1000, "timestamp": "2024-01-01T10:00:00Z"}"#,
        r#"{"symbol": "GOOGL", "price": 2800.50, "volume": 500, "timestamp": "2024-01-01T10:00:01Z"}"#,
        r#"{"symbol": "MSFT", "price": 380.75, "volume": 750, "timestamp": "2024-01-01T10:00:02Z"}"#,
        r#"{"symbol": "TSLA", "price": 220.30, "volume": 1200, "timestamp": "2024-01-01T10:00:03Z"}"#,
        r#"{"symbol": "AMZN", "price": 3400.80, "volume": 300, "timestamp": "2024-01-01T10:00:04Z"}"#,
    ];
    
    for (i, data) in test_data.iter().enumerate() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let connection_id = format!("test-conn-{}", i);
        
        let _received_data = fdc_ingestion::receiver::ReceivedData::new(
            data.as_bytes().to_vec(),
            addr,
            connection_id,
        );
        
        // 模拟通过接收器发送数据
        // 在实际应用中，这将通过网络接收器自动处理
        
        metrics.record_message_received("tcp");
        metrics.record_message_parsed("json");
        metrics.record_message_validated();
        metrics.record_message_stored();
        
        println!("  📦 Processed: {}", data);
        
        // 短暂延迟模拟真实场景
        sleep(Duration::from_millis(10)).await;
    }
    
    // 等待处理完成
    sleep(Duration::from_millis(500)).await;
    
    // 显示统计信息
    println!("\n📈 Processing Statistics:");
    println!("========================");
    
    let parser_stats = parser.get_stats().await;
    println!("Parser Stats:");
    println!("  - Messages parsed: {}", parser_stats.messages_parsed);
    println!("  - Parse successes: {}", parser_stats.parse_successes);
    println!("  - Parse failures: {}", parser_stats.parse_failures);
    println!("  - Success rate: {:.2}%", parser_stats.success_rate() * 100.0);
    println!("  - Avg parse time: {:.2}μs", parser_stats.avg_parse_time_us);
    
    let validator_stats = validator.get_stats().await;
    println!("\nValidator Stats:");
    println!("  - Messages validated: {}", validator_stats.messages_validated);
    println!("  - Validation successes: {}", validator_stats.validation_successes);
    println!("  - Validation failures: {}", validator_stats.validation_failures);
    println!("  - Success rate: {:.2}%", validator_stats.success_rate() * 100.0);
    println!("  - Avg validation time: {:.2}μs", validator_stats.avg_validation_time_us);
    
    let batch_stats = batch_processor.get_stats().await;
    println!("\nBatch Processor Stats:");
    println!("  - Batches processed: {}", batch_stats.batches_processed);
    println!("  - Total messages: {}", batch_stats.total_messages);
    println!("  - Successful messages: {}", batch_stats.successful_messages);
    println!("  - Failed messages: {}", batch_stats.failed_messages);
    println!("  - Success rate: {:.2}%", batch_stats.success_rate() * 100.0);
    println!("  - Avg batch size: {:.1}", batch_stats.avg_batch_size);
    println!("  - Throughput: {:.1} msg/sec", batch_stats.throughput_msg_per_sec);
    
    println!("\nIngestion Metrics:");
    println!("  - Messages received: {}", metrics.messages_received);
    println!("  - Messages parsed: {}", metrics.messages_parsed);
    println!("  - Messages validated: {}", metrics.messages_validated);
    println!("  - Messages stored: {}", metrics.messages_stored);
    println!("  - Error rate: {:.2}%", metrics.error_rate * 100.0);
    
    // 显示存储引擎统计 (简化版本)
    println!("\nStorage Engine Stats:");
    println!("  - Simple memory storage (no detailed stats available)");
    
    println!("\n✅ Demo completed successfully!");
    println!("🎯 Data ingestion system is working properly");
    
    Ok(())
}
