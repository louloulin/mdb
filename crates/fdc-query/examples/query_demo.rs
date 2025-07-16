//! High-Performance Query Engine demonstration example

use fdc_query::{
    engine::{QueryEngine, QueryEngineConfig},
    parser::{SqlParser, QueryType},
    optimizer::{QueryOptimizer, OptimizationRule},
    executor::{ExecutionContext, DefaultQueryExecutor, QueryExecutor},
    planner::QueryPlanner,
    cache::{QueryCache, CachePolicy},
    functions::BuiltinFunctions,
    aggregates::AggregateFunction,
    metrics::QueryMetrics,
    config::QueryConfig,
};
use fdc_storage::engines::memory::MemoryEngine;
use fdc_core::types::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 High-Performance Query Engine Demo");
    println!("=====================================");
    
    // 1. 演示SQL解析器
    demo_sql_parser().await?;
    
    // 2. 演示查询优化器
    demo_query_optimizer().await?;
    
    // 3. 演示查询计划器
    demo_query_planner().await?;
    
    // 4. 演示查询执行器
    demo_query_executor().await?;
    
    // 5. 演示查询引擎
    demo_query_engine().await?;
    
    // 6. 演示内置函数
    demo_builtin_functions().await?;
    
    // 7. 演示聚合函数
    demo_aggregate_functions().await?;
    
    // 8. 演示查询缓存
    demo_query_cache().await?;
    
    // 9. 演示查询指标
    demo_query_metrics().await?;
    
    // 10. 演示复杂查询
    demo_complex_queries().await?;
    
    println!("\n✅ All query engine demos completed successfully!");
    Ok(())
}

async fn demo_sql_parser() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 SQL Parser Demo");
    println!("------------------");
    
    let parser = SqlParser::new();
    
    // 解析简单SELECT查询
    let simple_query = "SELECT id, name, email FROM users WHERE age > 25";
    let parsed = parser.parse(simple_query)?;
    
    println!("📊 Simple SELECT query:");
    println!("  SQL: {}", simple_query);
    println!("  Type: {:?}", parsed.query_type);
    println!("  Tables: {:?}", parsed.tables);
    println!("  Is readonly: {}", parsed.is_readonly);
    
    // 解析JOIN查询
    let join_query = "SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id";
    let parsed_join = parser.parse(join_query)?;
    
    println!("\n📊 JOIN query:");
    println!("  SQL: {}", join_query);
    println!("  Type: {:?}", parsed_join.query_type);
    println!("  Tables: {:?}", parsed_join.tables);
    println!("  Multi-table: {}", parsed_join.is_multi_table());
    
    // 解析INSERT查询
    let insert_query = "INSERT INTO users (name, email) VALUES ('John', 'john@example.com')";
    let parsed_insert = parser.parse(insert_query)?;
    
    println!("\n📊 INSERT query:");
    println!("  SQL: {}", insert_query);
    println!("  Type: {:?}", parsed_insert.query_type);
    println!("  Tables: {:?}", parsed_insert.tables);
    println!("  Is readonly: {}", parsed_insert.is_readonly);
    
    // 验证SQL语法
    println!("\n✅ SQL validation:");
    println!("  Valid SQL: {}", parser.validate("SELECT * FROM users").is_ok());
    println!("  Invalid SQL: {}", parser.validate("INVALID SQL STATEMENT").is_err());
    
    // 格式化SQL
    let formatted = parser.format("select   *   from   users   where   id=1")?;
    println!("  Formatted SQL: {}", formatted);
    
    Ok(())
}

async fn demo_query_optimizer() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ Query Optimizer Demo");
    println!("----------------------");
    
    let mut optimizer = QueryOptimizer::new();
    
    // 设置表统计信息
    optimizer.set_table_stats("users".to_string(), 10000.0);
    optimizer.set_table_stats("orders".to_string(), 50000.0);
    
    println!("📊 Optimizer configuration:");
    println!("  Users table: {} rows", optimizer.get_table_stats("users").unwrap_or(0.0));
    println!("  Orders table: {} rows", optimizer.get_table_stats("orders").unwrap_or(0.0));
    
    // 解析并优化查询
    let parser = SqlParser::new();
    let query = parser.parse("SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id WHERE u.active = true")?;
    
    let optimized_plan = optimizer.optimize(query)?;
    
    println!("\n📈 Optimization results:");
    println!("  Applied rules: {}", optimized_plan.applied_rules.len());
    println!("  Rules: {:?}", optimized_plan.applied_rules);
    println!("  Original cost: {:.2}", optimized_plan.stats.original_cost);
    println!("  Optimized cost: {:.2}", optimized_plan.stats.optimized_cost);
    println!("  Cost reduction: {:.2}%", optimized_plan.stats.cost_reduction_percentage());
    println!("  Estimated time: {}ms", optimized_plan.estimated_time_ms);
    
    // 演示规则管理
    optimizer.disable_rule(&OptimizationRule::JoinReordering);
    println!("\n🔧 Rule management:");
    println!("  Disabled JOIN reordering rule");
    
    optimizer.enable_rule(OptimizationRule::SubqueryOptimization);
    println!("  Enabled subquery optimization rule");
    
    Ok(())
}

async fn demo_query_planner() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🗺️ Query Planner Demo");
    println!("---------------------");
    
    let mut planner = QueryPlanner::new();
    
    // 设置表统计信息
    planner.set_table_stats("users".to_string(), 10000, 200);
    planner.set_table_stats("orders".to_string(), 50000, 150);
    
    // 创建优化计划
    let parser = SqlParser::new();
    let query = parser.parse("SELECT * FROM users WHERE id = 1 ORDER BY name LIMIT 10")?;
    let mut optimizer = QueryOptimizer::new();
    let optimized_plan = optimizer.optimize(query)?;
    
    // 创建执行计划
    let execution_plan = planner.create_plan(&optimized_plan)?;
    
    println!("📊 Execution plan:");
    println!("  Root node: {:?}", execution_plan.root);
    println!("  Children: {}", execution_plan.children.len());
    println!("  Estimated cost: {:.2}", execution_plan.estimated_cost);
    println!("  Estimated rows: {}", execution_plan.estimated_rows);
    println!("  Plan depth: {}", execution_plan.depth());
    println!("  Total cost: {:.2}", execution_plan.total_cost());
    
    // 多表查询计划
    let join_query = parser.parse("SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id")?;
    let join_optimized = optimizer.optimize(join_query)?;
    let join_plan = planner.create_plan(&join_optimized)?;
    
    println!("\n📊 JOIN execution plan:");
    println!("  Root node: {:?}", join_plan.root);
    println!("  Children: {}", join_plan.children.len());
    println!("  Total cost: {:.2}", join_plan.total_cost());
    
    Ok(())
}

async fn demo_query_executor() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚙️ Query Executor Demo");
    println!("----------------------");
    
    // 创建内存存储引擎
    let memory_engine = MemoryEngine::new(HashMap::new()).await?;
    let executor = DefaultQueryExecutor::new(Arc::new(memory_engine));
    
    // 创建执行上下文
    let context = ExecutionContext::new("demo_query_1".to_string())
        .with_timeout(Duration::from_secs(30))
        .with_max_rows(1000);
    
    println!("📊 Execution context:");
    println!("  Query ID: {}", context.query_id);
    println!("  Timeout: {:?}", context.timeout);
    println!("  Max rows: {:?}", context.max_rows);
    println!("  Cache enabled: {}", context.enable_cache);
    
    // 执行SELECT查询
    let parser = SqlParser::new();
    let query = parser.parse("SELECT * FROM users WHERE age > 25")?;
    let mut optimizer = QueryOptimizer::new();
    let optimized_plan = optimizer.optimize(query)?;
    
    let result = executor.execute(optimized_plan, context).await?;
    
    println!("\n📈 Execution results:");
    println!("  Success: {}", result.is_success());
    println!("  Rows returned: {}", result.row_count());
    println!("  Execution time: {:.2}ms", result.execution_time_ms());
    println!("  Affected rows: {}", result.affected_rows);
    
    // 获取执行器统计信息
    let stats = executor.get_stats().await?;
    println!("\n📊 Executor statistics:");
    for (key, value) in stats {
        println!("  {}: {}", key, value);
    }
    
    Ok(())
}

async fn demo_query_engine() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔥 Query Engine Demo");
    println!("--------------------");
    
    // 创建查询引擎
    let memory_engine = MemoryEngine::new(HashMap::new()).await?;
    let config = QueryEngineConfig {
        enable_cache: true,
        cache_capacity: 1024 * 1024, // 1MB
        cache_policy: CachePolicy::LRU,
        query_timeout: Duration::from_secs(30),
        max_concurrent_queries: 100,
        enable_optimization: true,
        enable_metrics: true,
        ..Default::default()
    };
    
    let engine = QueryEngine::new(Arc::new(memory_engine), config);
    
    println!("📊 Engine configuration:");
    println!("  Cache enabled: {}", engine.config().enable_cache);
    println!("  Cache capacity: {} bytes", engine.config().cache_capacity);
    println!("  Optimization enabled: {}", engine.config().enable_optimization);
    println!("  Metrics enabled: {}", engine.config().enable_metrics);
    
    // 执行各种查询
    let queries = vec![
        "SELECT * FROM users",
        "SELECT * FROM users WHERE age > 25",
        "SELECT COUNT(*) FROM orders",
        "SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id",
        "INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')",
    ];
    
    println!("\n📈 Query execution results:");
    for (i, sql) in queries.iter().enumerate() {
        let result = engine.execute_sql(sql).await?;
        println!("  Query {}: {} ({}ms, {} rows)", 
            i + 1, 
            if result.is_success() { "SUCCESS" } else { "FAILED" },
            result.execution_time_ms(),
            result.row_count()
        );
    }
    
    // 获取缓存统计信息
    let cache_stats = engine.get_cache_stats().await?;
    println!("\n💾 Cache statistics:");
    println!("  Hits: {}", cache_stats.hits);
    println!("  Misses: {}", cache_stats.misses);
    println!("  Hit rate: {:.2}%", cache_stats.hit_rate() * 100.0);
    println!("  Size: {}/{}", cache_stats.size, cache_stats.capacity);
    
    // 获取查询统计信息
    let query_stats = engine.get_query_stats().await?;
    println!("\n📊 Query statistics:");
    println!("  Total queries: {}", query_stats.total_queries);
    println!("  Successful: {}", query_stats.successful_queries);
    println!("  Failed: {}", query_stats.failed_queries);
    println!("  Success rate: {:.2}%", query_stats.success_rate() * 100.0);
    println!("  Avg execution time: {:.2}μs", query_stats.avg_execution_time_us);
    
    Ok(())
}

async fn demo_builtin_functions() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 Built-in Functions Demo");
    println!("--------------------------");
    
    let functions = BuiltinFunctions::new();
    
    // 数学函数
    println!("📊 Math functions:");
    let abs_result = functions.call("ABS", &[Value::Int32(-42)])?;
    println!("  ABS(-42) = {:?}", abs_result);
    
    let round_result = functions.call("ROUND", &[Value::Float64(3.14159)])?;
    println!("  ROUND(3.14159) = {:?}", round_result);
    
    // 字符串函数
    println!("\n📊 String functions:");
    let upper_result = functions.call("UPPER", &[Value::String("hello world".to_string())])?;
    println!("  UPPER('hello world') = {:?}", upper_result);
    
    let lower_result = functions.call("LOWER", &[Value::String("HELLO WORLD".to_string())])?;
    println!("  LOWER('HELLO WORLD') = {:?}", lower_result);
    
    let length_result = functions.call("LENGTH", &[Value::String("test string".to_string())])?;
    println!("  LENGTH('test string') = {:?}", length_result);
    
    Ok(())
}

async fn demo_aggregate_functions() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📈 Aggregate Functions Demo");
    println!("---------------------------");
    
    let test_values = vec![
        Value::Int32(10),
        Value::Int32(20),
        Value::Int32(30),
        Value::Int32(40),
        Value::Int32(50),
    ];
    
    println!("📊 Test data: {:?}", test_values);
    
    // COUNT
    let count_result = AggregateFunction::Count.apply(&test_values)?;
    println!("  COUNT = {:?}", count_result);
    
    // SUM
    let sum_result = AggregateFunction::Sum.apply(&test_values)?;
    println!("  SUM = {:?}", sum_result);
    
    // AVG
    let avg_result = AggregateFunction::Avg.apply(&test_values)?;
    println!("  AVG = {:?}", avg_result);
    
    // MIN
    let min_result = AggregateFunction::Min.apply(&test_values)?;
    println!("  MIN = {:?}", min_result);
    
    // MAX
    let max_result = AggregateFunction::Max.apply(&test_values)?;
    println!("  MAX = {:?}", max_result);
    
    // FIRST
    let first_result = AggregateFunction::First.apply(&test_values)?;
    println!("  FIRST = {:?}", first_result);
    
    // LAST
    let last_result = AggregateFunction::Last.apply(&test_values)?;
    println!("  LAST = {:?}", last_result);
    
    Ok(())
}

async fn demo_query_cache() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 Query Cache Demo");
    println!("-------------------");
    
    let mut cache = QueryCache::new(CachePolicy::LRU, 3);
    
    println!("📊 Cache configuration:");
    println!("  Policy: LRU");
    println!("  Capacity: 3 entries");
    
    // 创建测试结果
    use fdc_query::executor::ExecutionResult;
    let result1 = ExecutionResult::success(Vec::new(), 1000);
    let result2 = ExecutionResult::success(Vec::new(), 2000);
    let result3 = ExecutionResult::success(Vec::new(), 3000);
    let result4 = ExecutionResult::success(Vec::new(), 4000);
    
    // 缓存操作
    cache.put("query1".to_string(), result1);
    cache.put("query2".to_string(), result2);
    cache.put("query3".to_string(), result3);
    
    println!("\n📈 Cache operations:");
    println!("  Added 3 queries to cache");
    
    // 缓存命中
    let hit = cache.get("query1");
    println!("  Get query1: {}", if hit.is_some() { "HIT" } else { "MISS" });
    
    // 缓存未命中
    let miss = cache.get("query_not_exists");
    println!("  Get query_not_exists: {}", if miss.is_some() { "HIT" } else { "MISS" });
    
    // 触发驱逐
    cache.put("query4".to_string(), result4);
    println!("  Added query4 (should trigger eviction)");
    
    // 获取统计信息
    let stats = cache.stats();
    println!("\n📊 Cache statistics:");
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
    println!("  Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!("  Evictions: {}", stats.evictions);
    println!("  Current size: {}/{}", stats.size, stats.capacity);
    
    Ok(())
}

async fn demo_query_metrics() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Query Metrics Demo");
    println!("---------------------");
    
    let mut metrics = QueryMetrics::new();
    
    // 模拟查询执行
    metrics.record_query_start();
    metrics.record_query_complete(Duration::from_millis(100), true);
    
    metrics.record_query_start();
    metrics.record_query_complete(Duration::from_millis(200), true);
    
    metrics.record_query_start();
    metrics.record_query_complete(Duration::from_millis(50), false);
    
    // 模拟缓存操作
    metrics.record_cache_hit();
    metrics.record_cache_hit();
    metrics.record_cache_miss();
    
    println!("📈 Query metrics:");
    println!("  Total queries: {}", metrics.total_queries);
    println!("  Successful: {}", metrics.successful_queries);
    println!("  Failed: {}", metrics.failed_queries);
    println!("  Success rate: {:.2}%", metrics.success_rate() * 100.0);
    println!("  Concurrent queries: {}", metrics.concurrent_queries);
    
    println!("\n⏱️ Performance metrics:");
    println!("  Avg execution time: {:.2}μs", metrics.avg_execution_time_us);
    println!("  Min execution time: {}μs", metrics.min_execution_time_us);
    println!("  Max execution time: {}μs", metrics.max_execution_time_us);
    println!("  Queries per second: {:.2}", metrics.queries_per_second());
    
    println!("\n💾 Cache metrics:");
    println!("  Cache hits: {}", metrics.cache_hits);
    println!("  Cache misses: {}", metrics.cache_misses);
    println!("  Cache hit rate: {:.2}%", metrics.cache_hit_rate() * 100.0);
    
    if let Some(uptime) = metrics.uptime() {
        println!("\n⏰ Uptime: {:.2}s", uptime.as_secs_f64());
    }
    
    Ok(())
}

async fn demo_complex_queries() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔥 Complex Queries Demo");
    println!("-----------------------");
    
    // 创建查询引擎
    let memory_engine = MemoryEngine::new(HashMap::new()).await?;
    let config = QueryEngineConfig::default();
    let engine = QueryEngine::new(Arc::new(memory_engine), config);
    
    // 复杂查询示例
    let complex_queries = vec![
        // 聚合查询
        "SELECT COUNT(*), AVG(amount) FROM orders WHERE created_at > '2024-01-01'",
        
        // 多表连接
        "SELECT u.name, u.email, COUNT(o.id) as order_count, SUM(o.amount) as total_amount 
         FROM users u 
         LEFT JOIN orders o ON u.id = o.user_id 
         WHERE u.active = true 
         GROUP BY u.id, u.name, u.email 
         HAVING COUNT(o.id) > 5 
         ORDER BY total_amount DESC 
         LIMIT 10",
        
        // 子查询
        "SELECT * FROM users WHERE id IN (
            SELECT DISTINCT user_id FROM orders WHERE amount > 1000
         )",
        
        // 窗口函数（简化）
        "SELECT name, amount, 
                ROW_NUMBER() OVER (ORDER BY amount DESC) as rank 
         FROM orders",
        
        // CTE（公共表表达式，简化）
        "WITH high_value_orders AS (
            SELECT user_id, SUM(amount) as total 
            FROM orders 
            WHERE amount > 500 
            GROUP BY user_id
         )
         SELECT u.name, hvo.total 
         FROM users u 
         JOIN high_value_orders hvo ON u.id = hvo.user_id",
    ];
    
    println!("📊 Executing complex queries:");
    
    for (i, sql) in complex_queries.iter().enumerate() {
        println!("\n🔍 Query {}: ", i + 1);
        println!("  SQL: {}", sql.lines().collect::<Vec<_>>().join(" ").trim());
        
        // 解析查询
        match engine.parse_sql(sql) {
            Ok(parsed) => {
                println!("  ✅ Parsing: SUCCESS");
                println!("  Type: {:?}", parsed.query_type);
                println!("  Tables: {:?}", parsed.tables);
                println!("  Multi-table: {}", parsed.is_multi_table());
            }
            Err(e) => {
                println!("  ❌ Parsing: FAILED - {}", e);
                continue;
            }
        }
        
        // 获取执行计划
        match engine.explain_query(sql).await {
            Ok(plan) => {
                println!("  📋 Plan: {:?}", plan.root);
                println!("  Cost: {:.2}", plan.estimated_cost);
                println!("  Rows: {}", plan.estimated_rows);
            }
            Err(e) => {
                println!("  ❌ Planning: FAILED - {}", e);
                continue;
            }
        }
        
        // 执行查询
        match engine.execute_sql(sql).await {
            Ok(result) => {
                println!("  ⚡ Execution: {} ({:.2}ms)", 
                    if result.is_success() { "SUCCESS" } else { "FAILED" },
                    result.execution_time_ms()
                );
                if result.is_success() {
                    println!("  Rows: {}", result.row_count());
                }
            }
            Err(e) => {
                println!("  ❌ Execution: FAILED - {}", e);
            }
        }
    }
    
    Ok(())
}
