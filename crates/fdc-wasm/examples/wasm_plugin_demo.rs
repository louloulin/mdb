//! WASM Plugin System demonstration example

use fdc_wasm::{
    runtime::{WasmRuntime, WasmRuntimeConfig},
    plugin::{WasmPlugin, PluginInfo, PluginType, PluginStatus},
    registry::PluginRegistry,
    security::SecurityPolicy,
    loader::PluginLoader,
    bridge::{WasmBridge, create_standard_host_functions},
    types::{WasmValue, WasmType, WasmFunctionSignature},
    events::{WasmEvent, WasmEventListener, ConsoleEventListener},
    metrics::WasmMetrics,
};
use std::path::PathBuf;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 WASM Plugin System Demo");
    println!("===========================");
    
    // 1. 演示WASM运行时
    demo_wasm_runtime()?;
    
    // 2. 演示安全策略
    demo_security_policy()?;
    
    // 3. 演示插件管理
    demo_plugin_management()?;
    
    // 4. 演示插件注册表
    demo_plugin_registry()?;
    
    // 5. 演示插件加载器
    demo_plugin_loader()?;
    
    // 6. 演示主机函数桥接
    demo_host_bridge()?;
    
    // 7. 演示事件系统
    demo_event_system()?;
    
    // 8. 演示指标收集
    demo_metrics_collection()?;
    
    println!("\n✅ All WASM plugin system demos completed successfully!");
    Ok(())
}

fn demo_wasm_runtime() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 WASM Runtime Demo");
    println!("-------------------");
    
    let config = WasmRuntimeConfig {
        memory_limit: 64 * 1024 * 1024, // 64MB
        execution_timeout_ms: 5000,
        max_plugins: 10,
        enable_wasi: false, // 暂时禁用WASI
        enable_threads: false,
        enable_simd: true,
        security_policy: SecurityPolicy::default(),
    };
    
    let runtime = WasmRuntime::new(config)?;
    
    println!("📋 Runtime configuration:");
    println!("  Memory limit: {} MB", runtime.config().memory_limit / (1024 * 1024));
    println!("  Execution timeout: {} ms", runtime.config().execution_timeout_ms);
    println!("  Max plugins: {}", runtime.config().max_plugins);
    println!("  WASI enabled: {}", runtime.config().enable_wasi);
    println!("  SIMD enabled: {}", runtime.config().enable_simd);
    
    // 创建一个简单的WASM模块（只有魔数和版本）
    let simple_wasm = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM魔数
        0x01, 0x00, 0x00, 0x00, // 版本1
    ];
    
    runtime.load_module("simple_module", &simple_wasm)?;
    println!("✅ Successfully loaded simple WASM module");
    
    println!("📊 Runtime status:");
    println!("  Module count: {}", runtime.module_count());
    println!("  Has module 'simple_module': {}", runtime.has_module("simple_module"));
    
    Ok(())
}

fn demo_security_policy() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔒 Security Policy Demo");
    println!("----------------------");
    
    // 默认策略
    let default_policy = SecurityPolicy::default();
    println!("📋 Default security policy:");
    println!("  Memory limit: {} MB", default_policy.memory_limit / (1024 * 1024));
    println!("  Execution timeout: {} ms", default_policy.execution_timeout_ms);
    println!("  Network access: {}", default_policy.network_access);
    println!("  File access: {}", default_policy.file_access);
    println!("  Sandbox enabled: {}", default_policy.sandbox_enabled);
    
    // 严格策略
    let strict_policy = SecurityPolicy::strict();
    println!("\n📋 Strict security policy:");
    println!("  Memory limit: {} MB", strict_policy.memory_limit / (1024 * 1024));
    println!("  Execution timeout: {} ms", strict_policy.execution_timeout_ms);
    println!("  CPU limit: {}%", strict_policy.cpu_limit_percent);
    
    // 宽松策略
    let permissive_policy = SecurityPolicy::permissive();
    println!("\n📋 Permissive security policy:");
    println!("  Memory limit: {} MB", permissive_policy.memory_limit / (1024 * 1024));
    println!("  Network access: {}", permissive_policy.network_access);
    println!("  File access: {}", permissive_policy.file_access);
    println!("  Allowed syscalls: {}", permissive_policy.allowed_syscalls.len());
    
    // 验证策略
    default_policy.validate()?;
    strict_policy.validate()?;
    permissive_policy.validate()?;
    println!("✅ All security policies validated successfully");
    
    Ok(())
}

fn demo_plugin_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔌 Plugin Management Demo");
    println!("------------------------");
    
    // 创建插件信息
    let mut plugin_info = PluginInfo::new(
        "data_transformer".to_string(),
        "1.0.0".to_string(),
        PluginType::DataTransform,
        PathBuf::from("/tmp/data_transformer.wasm"),
    );
    
    plugin_info.description = Some("A data transformation plugin".to_string());
    plugin_info.author = Some("FDC Team".to_string());
    plugin_info.add_exported_function("transform_data".to_string());
    plugin_info.add_dependency("fdc-core".to_string());
    plugin_info.add_permission("read_data".to_string());
    plugin_info.set_metadata("category".to_string(), "financial".to_string());
    
    println!("📋 Plugin information:");
    println!("  ID: {}", plugin_info.id);
    println!("  Name: {}", plugin_info.name);
    println!("  Version: {}", plugin_info.version);
    println!("  Type: {}", plugin_info.plugin_type);
    println!("  Status: {:?}", plugin_info.status);
    println!("  Description: {:?}", plugin_info.description);
    println!("  Author: {:?}", plugin_info.author);
    println!("  Exported functions: {:?}", plugin_info.exported_functions);
    println!("  Dependencies: {:?}", plugin_info.dependencies);
    println!("  Permissions: {:?}", plugin_info.permissions);
    
    // 创建WASM插件
    let wasm_bytes = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM魔数
        0x01, 0x00, 0x00, 0x00, // 版本1
    ];
    
    let security_policy = SecurityPolicy::default();
    let mut plugin = WasmPlugin::new(plugin_info, wasm_bytes, security_policy);
    
    // 设置配置
    plugin.set_config("input_format".to_string(), "json".to_string());
    plugin.set_config("output_format".to_string(), "binary".to_string());
    
    println!("\n📊 Plugin configuration:");
    for (key, value) in plugin.config() {
        println!("  {}: {}", key, value);
    }
    
    // 验证插件
    // plugin.validate()?; // 跳过验证，因为文件不存在
    println!("✅ Plugin created successfully");
    
    Ok(())
}

fn demo_plugin_registry() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📚 Plugin Registry Demo");
    println!("----------------------");
    
    let config = WasmRuntimeConfig::default();
    let registry = PluginRegistry::new(config)?;
    
    println!("📋 Registry status:");
    println!("  Plugin count: {}", registry.plugin_count());
    println!("  At capacity: {}", registry.is_at_capacity());
    
    // 列出插件
    let plugins = registry.list_plugins();
    println!("  Total plugins: {}", plugins.len());
    
    let data_transform_plugins = registry.list_plugins_by_type(PluginType::DataTransform);
    println!("  Data transform plugins: {}", data_transform_plugins.len());
    
    let loaded_plugins = registry.list_plugins_by_status(PluginStatus::Loaded);
    println!("  Loaded plugins: {}", loaded_plugins.len());
    
    // 获取指标
    let metrics = registry.metrics();
    println!("\n📊 Registry metrics:");
    println!("  Modules loaded: {}", metrics.modules_loaded);
    println!("  Function calls: {}", metrics.function_calls);
    println!("  Success rate: {:.2}%", metrics.success_rate() * 100.0);
    
    println!("✅ Registry demo completed");
    
    Ok(())
}

fn demo_plugin_loader() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📂 Plugin Loader Demo");
    println!("--------------------");
    
    let temp_dir = TempDir::new()?;
    let plugin_dir = temp_dir.path().to_path_buf();
    let security_policy = SecurityPolicy::default();
    
    let mut loader = PluginLoader::new(plugin_dir.clone(), security_policy);
    
    println!("📋 Loader configuration:");
    println!("  Plugin directory: {:?}", loader.plugin_dir());
    println!("  Hot reload enabled: {}", loader.is_hot_reload_enabled());
    
    // 启用热加载
    loader.enable_hot_reload()?;
    println!("  Hot reload enabled: {}", loader.is_hot_reload_enabled());
    
    // 扫描插件
    let configs = loader.scan_plugins()?;
    println!("📊 Scan results:");
    println!("  Found {} plugin configs", configs.len());
    
    // 创建插件模板
    loader.create_plugin_template("example_plugin", PluginType::CustomFunction)?;
    println!("✅ Created plugin template: example_plugin");
    
    // 再次扫描
    let configs = loader.scan_plugins()?;
    println!("📊 After template creation:");
    println!("  Found {} plugin configs", configs.len());
    
    // 检查文件更改
    let changes = loader.check_for_changes();
    println!("  File changes detected: {}", changes.len());
    
    println!("✅ Loader demo completed");
    
    Ok(())
}

fn demo_host_bridge() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌉 Host Bridge Demo");
    println!("------------------");
    
    let bridge = WasmBridge::new();
    
    println!("📋 Bridge status:");
    println!("  Function count: {}", bridge.function_count());
    
    // 注册标准主机函数
    let standard_functions = create_standard_host_functions();
    for function in standard_functions {
        bridge.register_host_function(function)?;
    }
    
    println!("📊 After registering standard functions:");
    println!("  Function count: {}", bridge.function_count());
    println!("  Available functions: {:?}", bridge.list_host_functions());
    
    // 调用主机函数
    let args = vec![WasmValue::F64(10.5), WasmValue::F64(5.5)];
    let result = bridge.call_host_function("add", &args)?;
    println!("📊 Function call result:");
    println!("  add(10.5, 5.5) = {:?}", result);
    
    // 获取函数统计
    if let Some(stats) = bridge.get_function_stats("add") {
        println!("📈 Function statistics:");
        println!("  Call count: {}", stats.call_count);
        println!("  Success count: {}", stats.success_count);
        println!("  Success rate: {:.2}%", stats.success_rate() * 100.0);
        println!("  Average execution time: {:.2}ms", stats.average_execution_time_ms);
    }
    
    println!("✅ Bridge demo completed");
    
    Ok(())
}

fn demo_event_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📡 Event System Demo");
    println!("-------------------");
    
    // 创建事件监听器
    let listener = ConsoleEventListener::new("demo_listener".to_string(), true);
    
    println!("📋 Event listener:");
    println!("  Name: {}", listener.name());
    
    // 创建一些事件
    let events = vec![
        WasmEvent::ModuleLoaded {
            module_name: "test_module".to_string(),
            module_size: 1024,
        },
        WasmEvent::FunctionCallStarted {
            module_name: "test_module".to_string(),
            function_name: "process_data".to_string(),
            args_count: 2,
        },
        WasmEvent::FunctionCallCompleted {
            module_name: "test_module".to_string(),
            function_name: "process_data".to_string(),
            execution_time_ms: 150,
            success: true,
        },
        WasmEvent::ModuleUnloaded {
            module_name: "test_module".to_string(),
        },
    ];
    
    println!("\n📊 Processing events:");
    for (i, event) in events.iter().enumerate() {
        println!("Event {}: {} (module: {:?})", 
            i + 1, 
            event.event_type(), 
            event.module_name()
        );
        listener.on_event(event.clone());
    }
    
    println!("✅ Event system demo completed");
    
    Ok(())
}

fn demo_metrics_collection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Metrics Collection Demo");
    println!("-------------------------");
    
    let mut metrics = WasmMetrics::new();
    
    // 记录一些指标
    metrics.record_module_loaded();
    metrics.record_module_loaded();
    
    metrics.record_module_function_call("module1", "func1", 100, true, 1024);
    metrics.record_module_function_call("module1", "func1", 150, true, 1536);
    metrics.record_module_function_call("module1", "func2", 200, false, 2048);
    metrics.record_module_function_call("module2", "func1", 80, true, 512);
    
    metrics.record_timeout();
    metrics.record_security_violation();
    metrics.record_hot_reload();
    
    println!("📈 Global metrics:");
    println!("  Modules loaded: {}", metrics.modules_loaded);
    println!("  Current modules: {}", metrics.current_modules);
    println!("  Function calls: {}", metrics.function_calls);
    println!("  Success rate: {:.2}%", metrics.success_rate() * 100.0);
    println!("  Average execution time: {:.2}ms", metrics.average_execution_time_ms);
    println!("  Peak memory usage: {} bytes", metrics.peak_memory_usage);
    println!("  Timeout count: {}", metrics.timeout_count);
    println!("  Security violations: {}", metrics.security_violations);
    println!("  Hot reload count: {}", metrics.hot_reload_count);
    
    if let Some(uptime) = metrics.uptime() {
        println!("  Uptime: {:.2}s", uptime.as_secs_f64());
        println!("  Calls per second: {:.2}", metrics.calls_per_second());
    }
    
    // 模块统计
    println!("\n📊 Module statistics:");
    for module in metrics.get_top_modules(5) {
        println!("  {}: {} calls ({:.2}% success)", 
            module.name, 
            module.function_calls,
            if module.function_calls > 0 { 
                module.successful_calls as f64 / module.function_calls as f64 * 100.0 
            } else { 
                0.0 
            }
        );
    }
    
    // 函数统计
    println!("\n📊 Function statistics:");
    for function in metrics.get_top_functions(5) {
        println!("  {}::{}: {} calls ({:.2}ms avg)", 
            function.module_name,
            function.name, 
            function.call_count,
            function.average_execution_time_ms
        );
    }
    
    // 生成报告
    println!("\n📋 Metrics Report:");
    println!("{}", metrics.generate_report());
    
    println!("✅ Metrics collection demo completed");
    
    Ok(())
}
