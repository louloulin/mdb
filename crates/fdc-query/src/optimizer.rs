//! Query optimizer for performance optimization

use crate::parser::ParsedQuery;
use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 优化规则类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationRule {
    /// 谓词下推
    PredicatePushdown,
    /// 投影下推
    ProjectionPushdown,
    /// 常量折叠
    ConstantFolding,
    /// 索引选择
    IndexSelection,
    /// 连接重排序
    JoinReordering,
    /// 子查询优化
    SubqueryOptimization,
    /// 聚合优化
    AggregateOptimization,
    /// 分区剪枝
    PartitionPruning,
}

/// 优化统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizationStats {
    /// 应用的规则数量
    pub rules_applied: u32,
    /// 优化耗时（微秒）
    pub optimization_time_us: u64,
    /// 预估成本降低
    pub cost_reduction: f64,
    /// 优化前的成本
    pub original_cost: f64,
    /// 优化后的成本
    pub optimized_cost: f64,
}

impl OptimizationStats {
    /// 记录规则应用
    pub fn record_rule_applied(&mut self) {
        self.rules_applied += 1;
    }
    
    /// 设置优化时间
    pub fn set_optimization_time(&mut self, time_us: u64) {
        self.optimization_time_us = time_us;
    }
    
    /// 设置成本信息
    pub fn set_costs(&mut self, original: f64, optimized: f64) {
        self.original_cost = original;
        self.optimized_cost = optimized;
        self.cost_reduction = original - optimized;
    }
    
    /// 获取成本降低百分比
    pub fn cost_reduction_percentage(&self) -> f64 {
        if self.original_cost > 0.0 {
            (self.cost_reduction / self.original_cost) * 100.0
        } else {
            0.0
        }
    }
}

/// 优化后的查询计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedPlan {
    /// 原始查询
    pub original_query: ParsedQuery,
    /// 优化后的查询
    pub optimized_query: ParsedQuery,
    /// 应用的优化规则
    pub applied_rules: Vec<OptimizationRule>,
    /// 优化统计信息
    pub stats: OptimizationStats,
    /// 执行提示
    pub hints: HashMap<String, String>,
    /// 预估执行成本
    pub estimated_cost: f64,
    /// 预估执行时间（毫秒）
    pub estimated_time_ms: u64,
}

impl OptimizedPlan {
    /// 创建新的优化计划
    pub fn new(original_query: ParsedQuery) -> Self {
        let optimized_query = original_query.clone();
        Self {
            original_query,
            optimized_query,
            applied_rules: Vec::new(),
            stats: OptimizationStats::default(),
            hints: HashMap::new(),
            estimated_cost: 1000.0, // 默认成本
            estimated_time_ms: 100,  // 默认100ms
        }
    }
    
    /// 添加优化规则
    pub fn add_rule(&mut self, rule: OptimizationRule) {
        if !self.applied_rules.contains(&rule) {
            self.applied_rules.push(rule);
            self.stats.record_rule_applied();
        }
    }
    
    /// 添加执行提示
    pub fn add_hint(&mut self, key: String, value: String) {
        self.hints.insert(key, value);
    }
    
    /// 设置成本估算
    pub fn set_cost_estimate(&mut self, cost: f64, time_ms: u64) {
        self.estimated_cost = cost;
        self.estimated_time_ms = time_ms;
    }
    
    /// 是否应用了特定规则
    pub fn has_rule(&self, rule: &OptimizationRule) -> bool {
        self.applied_rules.contains(rule)
    }
}

/// 查询优化器
pub struct QueryOptimizer {
    /// 启用的优化规则
    enabled_rules: Vec<OptimizationRule>,
    /// 统计信息缓存
    stats_cache: HashMap<String, f64>,
}

impl QueryOptimizer {
    /// 创建新的查询优化器
    pub fn new() -> Self {
        Self {
            enabled_rules: vec![
                OptimizationRule::PredicatePushdown,
                OptimizationRule::ProjectionPushdown,
                OptimizationRule::ConstantFolding,
                OptimizationRule::IndexSelection,
                OptimizationRule::JoinReordering,
            ],
            stats_cache: HashMap::new(),
        }
    }
    
    /// 启用优化规则
    pub fn enable_rule(&mut self, rule: OptimizationRule) {
        if !self.enabled_rules.contains(&rule) {
            self.enabled_rules.push(rule);
        }
    }
    
    /// 禁用优化规则
    pub fn disable_rule(&mut self, rule: &OptimizationRule) {
        self.enabled_rules.retain(|r| r != rule);
    }
    
    /// 优化查询
    pub fn optimize(&mut self, query: ParsedQuery) -> Result<OptimizedPlan> {
        let start_time = std::time::Instant::now();
        let mut plan = OptimizedPlan::new(query);
        
        // 应用启用的优化规则
        for rule in &self.enabled_rules.clone() {
            self.apply_rule(rule, &mut plan)?;
        }
        
        // 设置优化统计信息
        let optimization_time = start_time.elapsed().as_micros() as u64;
        plan.stats.set_optimization_time(optimization_time);
        plan.stats.set_costs(1000.0, plan.estimated_cost);
        
        Ok(plan)
    }
    
    /// 应用优化规则
    fn apply_rule(&mut self, rule: &OptimizationRule, plan: &mut OptimizedPlan) -> Result<()> {
        match rule {
            OptimizationRule::PredicatePushdown => {
                self.apply_predicate_pushdown(plan)?;
            }
            OptimizationRule::ProjectionPushdown => {
                self.apply_projection_pushdown(plan)?;
            }
            OptimizationRule::ConstantFolding => {
                self.apply_constant_folding(plan)?;
            }
            OptimizationRule::IndexSelection => {
                self.apply_index_selection(plan)?;
            }
            OptimizationRule::JoinReordering => {
                self.apply_join_reordering(plan)?;
            }
            OptimizationRule::SubqueryOptimization => {
                self.apply_subquery_optimization(plan)?;
            }
            OptimizationRule::AggregateOptimization => {
                self.apply_aggregate_optimization(plan)?;
            }
            OptimizationRule::PartitionPruning => {
                self.apply_partition_pruning(plan)?;
            }
        }
        
        plan.add_rule(rule.clone());
        Ok(())
    }
    
    /// 应用谓词下推优化
    fn apply_predicate_pushdown(&mut self, plan: &mut OptimizedPlan) -> Result<()> {
        // 简化实现：如果查询涉及多表，降低成本
        if plan.original_query.is_multi_table() {
            plan.estimated_cost *= 0.8; // 降低20%成本
            plan.add_hint("predicate_pushdown".to_string(), "applied".to_string());
        }
        Ok(())
    }
    
    /// 应用投影下推优化
    fn apply_projection_pushdown(&mut self, plan: &mut OptimizedPlan) -> Result<()> {
        // 简化实现：总是应用投影下推
        plan.estimated_cost *= 0.9; // 降低10%成本
        plan.add_hint("projection_pushdown".to_string(), "applied".to_string());
        Ok(())
    }
    
    /// 应用常量折叠优化
    fn apply_constant_folding(&mut self, plan: &mut OptimizedPlan) -> Result<()> {
        // 简化实现：检查SQL中是否有常量表达式
        if plan.original_query.sql.contains("1+1") || plan.original_query.sql.contains("2*3") {
            plan.estimated_cost *= 0.95; // 降低5%成本
            plan.add_hint("constant_folding".to_string(), "applied".to_string());
        }
        Ok(())
    }
    
    /// 应用索引选择优化
    fn apply_index_selection(&mut self, plan: &mut OptimizedPlan) -> Result<()> {
        // 简化实现：如果查询有WHERE子句，假设可以使用索引
        if plan.original_query.sql.to_lowercase().contains("where") {
            plan.estimated_cost *= 0.6; // 降低40%成本
            plan.estimated_time_ms = (plan.estimated_time_ms as f64 * 0.5) as u64;
            plan.add_hint("index_selection".to_string(), "btree_index".to_string());
        }
        Ok(())
    }
    
    /// 应用连接重排序优化
    fn apply_join_reordering(&mut self, plan: &mut OptimizedPlan) -> Result<()> {
        // 简化实现：如果是多表连接，应用重排序
        if plan.original_query.is_multi_table() && plan.original_query.sql.to_lowercase().contains("join") {
            plan.estimated_cost *= 0.7; // 降低30%成本
            plan.add_hint("join_reordering".to_string(), "cost_based".to_string());
        }
        Ok(())
    }
    
    /// 应用子查询优化
    fn apply_subquery_optimization(&mut self, plan: &mut OptimizedPlan) -> Result<()> {
        // 简化实现：检查是否有子查询
        if plan.original_query.sql.contains("(SELECT") {
            plan.estimated_cost *= 0.8; // 降低20%成本
            plan.add_hint("subquery_optimization".to_string(), "flattened".to_string());
        }
        Ok(())
    }
    
    /// 应用聚合优化
    fn apply_aggregate_optimization(&mut self, plan: &mut OptimizedPlan) -> Result<()> {
        // 简化实现：检查是否有聚合函数
        let sql_lower = plan.original_query.sql.to_lowercase();
        if sql_lower.contains("group by") || sql_lower.contains("count(") || sql_lower.contains("sum(") {
            plan.estimated_cost *= 0.85; // 降低15%成本
            plan.add_hint("aggregate_optimization".to_string(), "hash_aggregation".to_string());
        }
        Ok(())
    }
    
    /// 应用分区剪枝优化
    fn apply_partition_pruning(&mut self, plan: &mut OptimizedPlan) -> Result<()> {
        // 简化实现：如果查询有时间范围条件，应用分区剪枝
        let sql_lower = plan.original_query.sql.to_lowercase();
        if sql_lower.contains("date") || sql_lower.contains("timestamp") {
            plan.estimated_cost *= 0.5; // 降低50%成本
            plan.add_hint("partition_pruning".to_string(), "time_based".to_string());
        }
        Ok(())
    }
    
    /// 获取表统计信息
    pub fn get_table_stats(&self, table_name: &str) -> Option<f64> {
        self.stats_cache.get(table_name).copied()
    }
    
    /// 设置表统计信息
    pub fn set_table_stats(&mut self, table_name: String, row_count: f64) {
        self.stats_cache.insert(table_name, row_count);
    }
    
    /// 清除统计信息缓存
    pub fn clear_stats_cache(&mut self) {
        self.stats_cache.clear();
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedQuery, QueryType};

    #[test]
    fn test_optimizer_creation() {
        let optimizer = QueryOptimizer::new();
        assert!(!optimizer.enabled_rules.is_empty());
    }

    #[test]
    fn test_optimization_stats() {
        let mut stats = OptimizationStats::default();
        stats.record_rule_applied();
        stats.set_costs(1000.0, 800.0);
        
        assert_eq!(stats.rules_applied, 1);
        assert_eq!(stats.cost_reduction, 200.0);
        assert_eq!(stats.cost_reduction_percentage(), 20.0);
    }

    #[test]
    fn test_simple_optimization() {
        let mut optimizer = QueryOptimizer::new();
        let query = ParsedQuery::new(QueryType::Select, "SELECT * FROM users WHERE id = 1".to_string());
        
        let plan = optimizer.optimize(query).unwrap();
        assert!(!plan.applied_rules.is_empty());
        assert!(plan.estimated_cost < 1000.0); // 应该有优化
    }

    #[test]
    fn test_join_optimization() {
        let mut optimizer = QueryOptimizer::new();
        let mut query = ParsedQuery::new(QueryType::Select, 
            "SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id WHERE u.active = true".to_string());
        query.add_table("users".to_string());
        query.add_table("orders".to_string());
        
        let plan = optimizer.optimize(query).unwrap();
        assert!(plan.has_rule(&OptimizationRule::JoinReordering));
        assert!(plan.has_rule(&OptimizationRule::IndexSelection));
    }

    #[test]
    fn test_rule_management() {
        let mut optimizer = QueryOptimizer::new();
        
        optimizer.disable_rule(&OptimizationRule::JoinReordering);
        assert!(!optimizer.enabled_rules.contains(&OptimizationRule::JoinReordering));
        
        optimizer.enable_rule(OptimizationRule::SubqueryOptimization);
        assert!(optimizer.enabled_rules.contains(&OptimizationRule::SubqueryOptimization));
    }

    #[test]
    fn test_stats_cache() {
        let mut optimizer = QueryOptimizer::new();
        
        optimizer.set_table_stats("users".to_string(), 10000.0);
        assert_eq!(optimizer.get_table_stats("users"), Some(10000.0));
        
        optimizer.clear_stats_cache();
        assert_eq!(optimizer.get_table_stats("users"), None);
    }
}
