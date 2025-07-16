//! Query planner for creating execution plans

use crate::{parser::ParsedQuery, optimizer::OptimizedPlan};
use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 计划节点类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanNode {
    /// 表扫描
    TableScan { table: String, filters: Vec<String> },
    /// 索引扫描
    IndexScan { table: String, index: String, conditions: Vec<String> },
    /// 过滤
    Filter { condition: String },
    /// 投影
    Projection { columns: Vec<String> },
    /// 排序
    Sort { columns: Vec<String>, ascending: Vec<bool> },
    /// 限制
    Limit { count: usize, offset: usize },
    /// 聚合
    Aggregate { group_by: Vec<String>, aggregates: Vec<String> },
    /// 连接
    Join { join_type: JoinType, condition: String },
    /// 联合
    Union { all: bool },
}

/// 连接类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// 执行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// 根节点
    pub root: PlanNode,
    /// 子计划
    pub children: Vec<ExecutionPlan>,
    /// 预估成本
    pub estimated_cost: f64,
    /// 预估行数
    pub estimated_rows: u64,
    /// 计划属性
    pub properties: HashMap<String, String>,
}

impl ExecutionPlan {
    /// 创建新的执行计划
    pub fn new(root: PlanNode) -> Self {
        Self {
            root,
            children: Vec::new(),
            estimated_cost: 100.0,
            estimated_rows: 1000,
            properties: HashMap::new(),
        }
    }
    
    /// 添加子计划
    pub fn add_child(&mut self, child: ExecutionPlan) {
        self.children.push(child);
    }
    
    /// 设置成本估算
    pub fn set_estimates(&mut self, cost: f64, rows: u64) {
        self.estimated_cost = cost;
        self.estimated_rows = rows;
    }
    
    /// 添加属性
    pub fn add_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }
    
    /// 获取计划深度
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }
    
    /// 获取总成本
    pub fn total_cost(&self) -> f64 {
        self.estimated_cost + self.children.iter().map(|c| c.total_cost()).sum::<f64>()
    }
}

/// 查询计划器
pub struct QueryPlanner {
    /// 统计信息
    table_stats: HashMap<String, TableStats>,
}

/// 表统计信息
#[derive(Debug, Clone)]
struct TableStats {
    row_count: u64,
    avg_row_size: usize,
    indexes: Vec<String>,
}

impl QueryPlanner {
    /// 创建新的查询计划器
    pub fn new() -> Self {
        Self {
            table_stats: HashMap::new(),
        }
    }
    
    /// 创建执行计划
    pub fn create_plan(&self, optimized_plan: &OptimizedPlan) -> Result<ExecutionPlan> {
        let query = &optimized_plan.original_query;
        
        match query.query_type {
            crate::parser::QueryType::Select => self.create_select_plan(query),
            crate::parser::QueryType::Insert => self.create_insert_plan(query),
            crate::parser::QueryType::Update => self.create_update_plan(query),
            crate::parser::QueryType::Delete => self.create_delete_plan(query),
            _ => Err(Error::unimplemented("Query type not supported for planning")),
        }
    }
    
    /// 创建SELECT执行计划
    fn create_select_plan(&self, query: &ParsedQuery) -> Result<ExecutionPlan> {
        let mut plan = if query.tables.len() == 1 {
            // 单表查询
            self.create_single_table_plan(&query.tables[0], query)?
        } else {
            // 多表查询
            self.create_multi_table_plan(&query.tables, query)?
        };
        
        // 添加排序
        if query.sql.to_lowercase().contains("order by") {
            let sort_node = PlanNode::Sort {
                columns: vec!["id".to_string()], // 简化实现
                ascending: vec![true],
            };
            let mut sort_plan = ExecutionPlan::new(sort_node);
            sort_plan.add_child(plan);
            plan = sort_plan;
        }
        
        // 添加限制
        if query.sql.to_lowercase().contains("limit") {
            let limit_node = PlanNode::Limit {
                count: 100, // 简化实现
                offset: 0,
            };
            let mut limit_plan = ExecutionPlan::new(limit_node);
            limit_plan.add_child(plan);
            plan = limit_plan;
        }
        
        Ok(plan)
    }
    
    /// 创建单表计划
    fn create_single_table_plan(&self, table: &str, query: &ParsedQuery) -> Result<ExecutionPlan> {
        let scan_node = if query.sql.to_lowercase().contains("where") {
            // 尝试使用索引扫描
            if self.has_suitable_index(table, &query.sql) {
                PlanNode::IndexScan {
                    table: table.to_string(),
                    index: "primary".to_string(),
                    conditions: vec!["id = ?".to_string()],
                }
            } else {
                PlanNode::TableScan {
                    table: table.to_string(),
                    filters: vec!["condition".to_string()],
                }
            }
        } else {
            PlanNode::TableScan {
                table: table.to_string(),
                filters: Vec::new(),
            }
        };
        
        let mut plan = ExecutionPlan::new(scan_node);
        
        // 设置成本估算
        let stats = self.get_table_stats(table);
        plan.set_estimates(stats.row_count as f64 * 0.1, stats.row_count);
        
        Ok(plan)
    }
    
    /// 创建多表计划
    fn create_multi_table_plan(&self, tables: &[String], query: &ParsedQuery) -> Result<ExecutionPlan> {
        if tables.len() < 2 {
            return Err(Error::validation("Multi-table plan requires at least 2 tables"));
        }
        
        // 创建第一个表的扫描计划
        let left_plan = self.create_single_table_plan(&tables[0], query)?;
        
        // 创建第二个表的扫描计划
        let right_plan = self.create_single_table_plan(&tables[1], query)?;
        
        // 创建连接计划
        let join_node = PlanNode::Join {
            join_type: JoinType::Inner,
            condition: "left.id = right.id".to_string(),
        };
        
        let mut join_plan = ExecutionPlan::new(join_node);
        join_plan.add_child(left_plan);
        join_plan.add_child(right_plan);
        
        // 设置连接成本
        join_plan.set_estimates(1000.0, 500);
        
        Ok(join_plan)
    }
    
    /// 创建INSERT执行计划
    fn create_insert_plan(&self, query: &ParsedQuery) -> Result<ExecutionPlan> {
        if let Some(table) = query.primary_table() {
            let insert_node = PlanNode::TableScan {
                table: table.clone(),
                filters: Vec::new(),
            };
            let plan = ExecutionPlan::new(insert_node);
            Ok(plan)
        } else {
            Err(Error::validation("INSERT query must specify a table"))
        }
    }
    
    /// 创建UPDATE执行计划
    fn create_update_plan(&self, query: &ParsedQuery) -> Result<ExecutionPlan> {
        if let Some(table) = query.primary_table() {
            let update_node = PlanNode::TableScan {
                table: table.clone(),
                filters: vec!["update_condition".to_string()],
            };
            let plan = ExecutionPlan::new(update_node);
            Ok(plan)
        } else {
            Err(Error::validation("UPDATE query must specify a table"))
        }
    }
    
    /// 创建DELETE执行计划
    fn create_delete_plan(&self, query: &ParsedQuery) -> Result<ExecutionPlan> {
        if let Some(table) = query.primary_table() {
            let delete_node = PlanNode::TableScan {
                table: table.clone(),
                filters: vec!["delete_condition".to_string()],
            };
            let plan = ExecutionPlan::new(delete_node);
            Ok(plan)
        } else {
            Err(Error::validation("DELETE query must specify a table"))
        }
    }
    
    /// 检查是否有合适的索引
    fn has_suitable_index(&self, table: &str, _sql: &str) -> bool {
        self.table_stats.get(table)
            .map(|stats| !stats.indexes.is_empty())
            .unwrap_or(false)
    }
    
    /// 获取表统计信息
    fn get_table_stats(&self, table: &str) -> TableStats {
        self.table_stats.get(table).cloned().unwrap_or(TableStats {
            row_count: 1000,
            avg_row_size: 100,
            indexes: vec!["primary".to_string()],
        })
    }
    
    /// 设置表统计信息
    pub fn set_table_stats(&mut self, table: String, row_count: u64, avg_row_size: usize) {
        self.table_stats.insert(table, TableStats {
            row_count,
            avg_row_size,
            indexes: vec!["primary".to_string()],
        });
    }
}

impl Default for QueryPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedQuery, QueryType};
    use crate::optimizer::OptimizedPlan;

    #[test]
    fn test_execution_plan() {
        let node = PlanNode::TableScan {
            table: "users".to_string(),
            filters: Vec::new(),
        };
        let mut plan = ExecutionPlan::new(node);
        plan.set_estimates(100.0, 1000);
        
        assert_eq!(plan.estimated_cost, 100.0);
        assert_eq!(plan.estimated_rows, 1000);
        assert_eq!(plan.depth(), 1);
    }

    #[test]
    fn test_planner_creation() {
        let planner = QueryPlanner::new();
        assert!(planner.table_stats.is_empty());
    }

    #[test]
    fn test_single_table_plan() {
        let planner = QueryPlanner::new();
        let mut query = ParsedQuery::new(QueryType::Select, "SELECT * FROM users".to_string());
        query.add_table("users".to_string());
        
        let optimized_plan = OptimizedPlan::new(query);
        let plan = planner.create_plan(&optimized_plan).unwrap();
        
        assert!(matches!(plan.root, PlanNode::TableScan { .. }));
    }

    #[test]
    fn test_multi_table_plan() {
        let planner = QueryPlanner::new();
        let mut query = ParsedQuery::new(QueryType::Select, 
            "SELECT * FROM users u JOIN orders o ON u.id = o.user_id".to_string());
        query.add_table("users".to_string());
        query.add_table("orders".to_string());
        
        let optimized_plan = OptimizedPlan::new(query);
        let plan = planner.create_plan(&optimized_plan).unwrap();
        
        assert!(matches!(plan.root, PlanNode::Join { .. }));
        assert_eq!(plan.children.len(), 2);
    }

    #[test]
    fn test_table_stats() {
        let mut planner = QueryPlanner::new();
        planner.set_table_stats("users".to_string(), 10000, 200);
        
        let stats = planner.get_table_stats("users");
        assert_eq!(stats.row_count, 10000);
        assert_eq!(stats.avg_row_size, 200);
    }
}
