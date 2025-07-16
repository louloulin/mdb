//! WASM security and sandboxing

use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

/// WASM安全策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// 内存限制（字节）
    pub memory_limit: usize,
    /// 执行超时（毫秒）
    pub execution_timeout_ms: u64,
    /// 允许的系统调用
    pub allowed_syscalls: HashSet<String>,
    /// 是否允许网络访问
    pub network_access: bool,
    /// 是否允许文件访问
    pub file_access: bool,
    /// 是否允许环境变量访问
    pub env_access: bool,
    /// 允许的文件路径
    pub allowed_paths: HashSet<String>,
    /// 允许的网络地址
    pub allowed_hosts: HashSet<String>,
    /// 最大文件大小
    pub max_file_size: usize,
    /// 最大网络连接数
    pub max_connections: usize,
    /// CPU使用限制（百分比）
    pub cpu_limit_percent: f64,
    /// 是否启用沙箱
    pub sandbox_enabled: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            memory_limit: 128 * 1024 * 1024, // 128MB
            execution_timeout_ms: 5000,       // 5秒
            allowed_syscalls: HashSet::new(),
            network_access: false,
            file_access: false,
            env_access: false,
            allowed_paths: HashSet::new(),
            allowed_hosts: HashSet::new(),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_connections: 0,
            cpu_limit_percent: 50.0,
            sandbox_enabled: true,
        }
    }
}

impl SecurityPolicy {
    /// 创建严格的安全策略
    pub fn strict() -> Self {
        Self {
            memory_limit: 64 * 1024 * 1024, // 64MB
            execution_timeout_ms: 1000,      // 1秒
            allowed_syscalls: HashSet::new(),
            network_access: false,
            file_access: false,
            env_access: false,
            allowed_paths: HashSet::new(),
            allowed_hosts: HashSet::new(),
            max_file_size: 1 * 1024 * 1024, // 1MB
            max_connections: 0,
            cpu_limit_percent: 25.0,
            sandbox_enabled: true,
        }
    }
    
    /// 创建宽松的安全策略
    pub fn permissive() -> Self {
        let mut allowed_syscalls = HashSet::new();
        allowed_syscalls.insert("read".to_string());
        allowed_syscalls.insert("write".to_string());
        allowed_syscalls.insert("open".to_string());
        allowed_syscalls.insert("close".to_string());
        
        let mut allowed_paths = HashSet::new();
        allowed_paths.insert("/tmp".to_string());
        allowed_paths.insert("/var/tmp".to_string());
        
        Self {
            memory_limit: 512 * 1024 * 1024, // 512MB
            execution_timeout_ms: 30000,      // 30秒
            allowed_syscalls,
            network_access: true,
            file_access: true,
            env_access: true,
            allowed_paths,
            allowed_hosts: HashSet::new(),
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_connections: 10,
            cpu_limit_percent: 80.0,
            sandbox_enabled: true,
        }
    }
    
    /// 添加允许的系统调用
    pub fn allow_syscall(&mut self, syscall: &str) {
        self.allowed_syscalls.insert(syscall.to_string());
    }
    
    /// 添加允许的文件路径
    pub fn allow_path(&mut self, path: &str) {
        self.allowed_paths.insert(path.to_string());
    }
    
    /// 添加允许的网络主机
    pub fn allow_host(&mut self, host: &str) {
        self.allowed_hosts.insert(host.to_string());
    }
    
    /// 检查系统调用是否被允许
    pub fn is_syscall_allowed(&self, syscall: &str) -> bool {
        self.allowed_syscalls.contains(syscall)
    }
    
    /// 检查文件路径是否被允许
    pub fn is_path_allowed(&self, path: &str) -> bool {
        if !self.file_access {
            return false;
        }
        
        if self.allowed_paths.is_empty() {
            return true; // 如果没有限制，则允许所有路径
        }
        
        self.allowed_paths.iter().any(|allowed_path| {
            path.starts_with(allowed_path)
        })
    }
    
    /// 检查网络主机是否被允许
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.network_access {
            return false;
        }
        
        if self.allowed_hosts.is_empty() {
            return true; // 如果没有限制，则允许所有主机
        }
        
        self.allowed_hosts.contains(host)
    }
    
    /// 验证策略配置
    pub fn validate(&self) -> Result<()> {
        if self.memory_limit == 0 {
            return Err(Error::validation("Memory limit must be greater than 0"));
        }
        
        if self.execution_timeout_ms == 0 {
            return Err(Error::validation("Execution timeout must be greater than 0"));
        }
        
        if self.max_file_size == 0 {
            return Err(Error::validation("Max file size must be greater than 0"));
        }
        
        if !(0.0..=100.0).contains(&self.cpu_limit_percent) {
            return Err(Error::validation("CPU limit must be between 0 and 100"));
        }
        
        Ok(())
    }
    
    /// 获取执行超时时长
    pub fn execution_timeout(&self) -> Duration {
        Duration::from_millis(self.execution_timeout_ms)
    }
}

/// 安全违规类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityViolationType {
    /// 内存限制超出
    MemoryLimitExceeded,
    /// 执行超时
    ExecutionTimeout,
    /// 未授权的系统调用
    UnauthorizedSyscall(String),
    /// 未授权的文件访问
    UnauthorizedFileAccess(String),
    /// 未授权的网络访问
    UnauthorizedNetworkAccess(String),
    /// 未授权的环境变量访问
    UnauthorizedEnvAccess(String),
    /// 文件大小超限
    FileSizeExceeded(usize),
    /// 连接数超限
    ConnectionLimitExceeded,
    /// CPU使用超限
    CpuLimitExceeded,
    /// 沙箱逃逸尝试
    SandboxEscape,
}

/// 安全违规事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    /// 违规类型
    pub violation_type: SecurityViolationType,
    /// 插件名称
    pub plugin_name: String,
    /// 违规时间
    pub timestamp: std::time::SystemTime,
    /// 详细信息
    pub details: String,
    /// 严重程度
    pub severity: SecuritySeverity,
}

/// 安全严重程度
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecuritySeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

impl SecurityViolation {
    /// 创建新的安全违规事件
    pub fn new(
        violation_type: SecurityViolationType,
        plugin_name: String,
        details: String,
    ) -> Self {
        let severity = match violation_type {
            SecurityViolationType::MemoryLimitExceeded => SecuritySeverity::Error,
            SecurityViolationType::ExecutionTimeout => SecuritySeverity::Warning,
            SecurityViolationType::UnauthorizedSyscall(_) => SecuritySeverity::Critical,
            SecurityViolationType::UnauthorizedFileAccess(_) => SecuritySeverity::Error,
            SecurityViolationType::UnauthorizedNetworkAccess(_) => SecuritySeverity::Error,
            SecurityViolationType::UnauthorizedEnvAccess(_) => SecuritySeverity::Warning,
            SecurityViolationType::FileSizeExceeded(_) => SecuritySeverity::Warning,
            SecurityViolationType::ConnectionLimitExceeded => SecuritySeverity::Warning,
            SecurityViolationType::CpuLimitExceeded => SecuritySeverity::Warning,
            SecurityViolationType::SandboxEscape => SecuritySeverity::Critical,
        };
        
        Self {
            violation_type,
            plugin_name,
            timestamp: std::time::SystemTime::now(),
            details,
            severity,
        }
    }
    
    /// 检查是否为严重违规
    pub fn is_critical(&self) -> bool {
        self.severity == SecuritySeverity::Critical
    }
    
    /// 检查是否需要立即停止插件
    pub fn should_stop_plugin(&self) -> bool {
        matches!(
            self.violation_type,
            SecurityViolationType::UnauthorizedSyscall(_)
                | SecurityViolationType::SandboxEscape
                | SecurityViolationType::MemoryLimitExceeded
        )
    }
}

/// 安全监控器
#[derive(Debug)]
pub struct SecurityMonitor {
    /// 安全策略
    policy: SecurityPolicy,
    /// 违规历史
    violations: Vec<SecurityViolation>,
    /// 是否启用监控
    enabled: bool,
}

impl SecurityMonitor {
    /// 创建新的安全监控器
    pub fn new(policy: SecurityPolicy) -> Self {
        Self {
            policy,
            violations: Vec::new(),
            enabled: true,
        }
    }
    
    /// 检查内存使用
    pub fn check_memory_usage(&mut self, plugin_name: &str, memory_usage: usize) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        if memory_usage > self.policy.memory_limit {
            let violation = SecurityViolation::new(
                SecurityViolationType::MemoryLimitExceeded,
                plugin_name.to_string(),
                format!("Memory usage {} exceeds limit {}", memory_usage, self.policy.memory_limit),
            );
            
            self.violations.push(violation.clone());
            
            return Err(Error::plugin(format!(
                "Memory limit exceeded for plugin {}: {} > {}",
                plugin_name, memory_usage, self.policy.memory_limit
            )));
        }
        
        Ok(())
    }
    
    /// 检查执行时间
    pub fn check_execution_time(&mut self, plugin_name: &str, execution_time_ms: u64) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        if execution_time_ms > self.policy.execution_timeout_ms {
            let violation = SecurityViolation::new(
                SecurityViolationType::ExecutionTimeout,
                plugin_name.to_string(),
                format!("Execution time {} ms exceeds timeout {} ms", execution_time_ms, self.policy.execution_timeout_ms),
            );
            
            self.violations.push(violation.clone());
            
            return Err(Error::plugin(format!(
                "Execution timeout for plugin {}: {} ms > {} ms",
                plugin_name, execution_time_ms, self.policy.execution_timeout_ms
            )));
        }
        
        Ok(())
    }
    
    /// 检查文件访问
    pub fn check_file_access(&mut self, plugin_name: &str, path: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        if !self.policy.is_path_allowed(path) {
            let violation = SecurityViolation::new(
                SecurityViolationType::UnauthorizedFileAccess(path.to_string()),
                plugin_name.to_string(),
                format!("Unauthorized file access: {}", path),
            );
            
            self.violations.push(violation.clone());
            
            return Err(Error::plugin(format!(
                "Unauthorized file access for plugin {}: {}",
                plugin_name, path
            )));
        }
        
        Ok(())
    }
    
    /// 获取违规历史
    pub fn violations(&self) -> &[SecurityViolation] {
        &self.violations
    }
    
    /// 清除违规历史
    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }
    
    /// 启用/禁用监控
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 获取安全策略
    pub fn policy(&self) -> &SecurityPolicy {
        &self.policy
    }
    
    /// 更新安全策略
    pub fn update_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        policy.validate()?;
        self.policy = policy;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_policy_default() {
        let policy = SecurityPolicy::default();
        assert_eq!(policy.memory_limit, 128 * 1024 * 1024);
        assert_eq!(policy.execution_timeout_ms, 5000);
        assert!(!policy.network_access);
        assert!(!policy.file_access);
        assert!(policy.sandbox_enabled);
    }

    #[test]
    fn test_security_policy_strict() {
        let policy = SecurityPolicy::strict();
        assert_eq!(policy.memory_limit, 64 * 1024 * 1024);
        assert_eq!(policy.execution_timeout_ms, 1000);
        assert_eq!(policy.cpu_limit_percent, 25.0);
    }

    #[test]
    fn test_security_policy_permissive() {
        let policy = SecurityPolicy::permissive();
        assert!(policy.network_access);
        assert!(policy.file_access);
        assert!(policy.env_access);
        assert!(!policy.allowed_syscalls.is_empty());
    }

    #[test]
    fn test_security_policy_validation() {
        let mut policy = SecurityPolicy::default();
        assert!(policy.validate().is_ok());
        
        policy.memory_limit = 0;
        assert!(policy.validate().is_err());
        
        policy.memory_limit = 1024;
        policy.cpu_limit_percent = 150.0;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_security_violation() {
        let violation = SecurityViolation::new(
            SecurityViolationType::MemoryLimitExceeded,
            "test_plugin".to_string(),
            "Memory limit exceeded".to_string(),
        );
        
        assert_eq!(violation.plugin_name, "test_plugin");
        assert_eq!(violation.severity, SecuritySeverity::Error);
        assert!(!violation.is_critical());
        assert!(violation.should_stop_plugin());
    }

    #[test]
    fn test_security_monitor() {
        let policy = SecurityPolicy::default();
        let mut monitor = SecurityMonitor::new(policy);
        
        // 测试内存检查
        let result = monitor.check_memory_usage("test_plugin", 64 * 1024 * 1024);
        assert!(result.is_ok());
        
        let result = monitor.check_memory_usage("test_plugin", 256 * 1024 * 1024);
        assert!(result.is_err());
        assert_eq!(monitor.violations().len(), 1);
        
        // 测试执行时间检查
        let result = monitor.check_execution_time("test_plugin", 1000);
        assert!(result.is_ok());
        
        let result = monitor.check_execution_time("test_plugin", 10000);
        assert!(result.is_err());
        assert_eq!(monitor.violations().len(), 2);
    }
}
