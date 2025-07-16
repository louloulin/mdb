//! SQL parser for query engine

use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlparser::{ast::Statement, dialect::GenericDialect, parser::Parser};
use std::collections::HashMap;

/// 查询类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Create,
    Drop,
    Alter,
    Show,
    Describe,
    Explain,
}

/// 解析后的查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedQuery {
    /// 查询类型
    pub query_type: QueryType,
    /// 原始SQL
    pub sql: String,
    /// 解析后的AST
    pub ast: String, // 简化为字符串表示
    /// 涉及的表
    pub tables: Vec<String>,
    /// 查询参数
    pub parameters: HashMap<String, String>,
    /// 是否只读查询
    pub is_readonly: bool,
}

impl ParsedQuery {
    /// 创建新的解析查询
    pub fn new(query_type: QueryType, sql: String) -> Self {
        Self {
            query_type,
            sql,
            ast: String::new(),
            tables: Vec::new(),
            parameters: HashMap::new(),
            is_readonly: matches!(query_type, QueryType::Select | QueryType::Show | QueryType::Describe | QueryType::Explain),
        }
    }
    
    /// 添加表名
    pub fn add_table(&mut self, table: String) {
        if !self.tables.contains(&table) {
            self.tables.push(table);
        }
    }
    
    /// 添加参数
    pub fn add_parameter(&mut self, key: String, value: String) {
        self.parameters.insert(key, value);
    }
    
    /// 获取主表名
    pub fn primary_table(&self) -> Option<&String> {
        self.tables.first()
    }
    
    /// 是否涉及多表
    pub fn is_multi_table(&self) -> bool {
        self.tables.len() > 1
    }
}

/// SQL解析器
pub struct SqlParser {
    dialect: GenericDialect,
}

impl SqlParser {
    /// 创建新的SQL解析器
    pub fn new() -> Self {
        Self {
            dialect: GenericDialect {},
        }
    }
    
    /// 解析SQL语句
    pub fn parse(&self, sql: &str) -> Result<ParsedQuery> {
        // 使用sqlparser解析SQL
        let statements = Parser::parse_sql(&self.dialect, sql)
            .map_err(|e| Error::validation(format!("SQL parsing failed: {}", e)))?;
        
        if statements.is_empty() {
            return Err(Error::validation("Empty SQL statement"));
        }
        
        if statements.len() > 1 {
            return Err(Error::validation("Multiple statements not supported"));
        }
        
        let statement = &statements[0];
        let mut parsed_query = self.analyze_statement(statement, sql)?;
        
        // 设置AST字符串表示
        parsed_query.ast = format!("{:?}", statement);
        
        Ok(parsed_query)
    }
    
    /// 分析SQL语句
    fn analyze_statement(&self, statement: &Statement, sql: &str) -> Result<ParsedQuery> {
        match statement {
            Statement::Query(query) => {
                let mut parsed = ParsedQuery::new(QueryType::Select, sql.to_string());
                self.extract_tables_from_query(query, &mut parsed);
                Ok(parsed)
            }
            Statement::Insert { .. } => {
                let mut parsed = ParsedQuery::new(QueryType::Insert, sql.to_string());
                // 简化实现：从SQL中提取表名
                if let Some(table_name) = self.extract_table_from_sql(sql, "INSERT INTO") {
                    parsed.add_table(table_name);
                }
                Ok(parsed)
            }
            Statement::Update { table, .. } => {
                let mut parsed = ParsedQuery::new(QueryType::Update, sql.to_string());
                parsed.add_table(table.to_string());
                Ok(parsed)
            }
            Statement::Delete { .. } => {
                let mut parsed = ParsedQuery::new(QueryType::Delete, sql.to_string());
                // 简化实现：从SQL中提取表名
                if let Some(table_name) = self.extract_table_from_sql(sql, "DELETE FROM") {
                    parsed.add_table(table_name);
                }
                Ok(parsed)
            }
            Statement::CreateTable { .. } => {
                let mut parsed = ParsedQuery::new(QueryType::Create, sql.to_string());
                // 简化实现：从SQL中提取表名
                if let Some(table_name) = self.extract_table_from_sql(sql, "CREATE TABLE") {
                    parsed.add_table(table_name);
                }
                Ok(parsed)
            }
            Statement::Drop { .. } => {
                let mut parsed = ParsedQuery::new(QueryType::Drop, sql.to_string());
                // 简化实现：从SQL中提取表名
                if let Some(table_name) = self.extract_table_from_sql(sql, "DROP TABLE") {
                    parsed.add_table(table_name);
                }
                Ok(parsed)
            }
            Statement::ShowTables { .. } => {
                Ok(ParsedQuery::new(QueryType::Show, sql.to_string()))
            }
            _ => {
                // 其他语句类型的简化处理
                Ok(ParsedQuery::new(QueryType::Select, sql.to_string()))
            }
        }
    }
    
    /// 从查询中提取表名
    fn extract_tables_from_query(&self, query: &sqlparser::ast::Query, parsed: &mut ParsedQuery) {
        if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
            for table_with_joins in &select.from {
                self.extract_table_name(&table_with_joins.relation, parsed);
                
                // 处理JOIN
                for join in &table_with_joins.joins {
                    self.extract_table_name(&join.relation, parsed);
                }
            }
        }
    }
    
    /// 提取表名
    fn extract_table_name(&self, table_factor: &sqlparser::ast::TableFactor, parsed: &mut ParsedQuery) {
        match table_factor {
            sqlparser::ast::TableFactor::Table { name, .. } => {
                parsed.add_table(name.to_string());
            }
            sqlparser::ast::TableFactor::Derived { .. } => {
                // 子查询，暂时忽略
            }
            _ => {
                // 其他类型暂时忽略
            }
        }
    }
    
    /// 验证SQL语法
    pub fn validate(&self, sql: &str) -> Result<()> {
        Parser::parse_sql(&self.dialect, sql)
            .map_err(|e| Error::validation(format!("SQL validation failed: {}", e)))?;
        Ok(())
    }
    
    /// 格式化SQL
    pub fn format(&self, sql: &str) -> Result<String> {
        let statements = Parser::parse_sql(&self.dialect, sql)
            .map_err(|e| Error::validation(format!("SQL parsing failed: {}", e)))?;
        
        if statements.is_empty() {
            return Ok(String::new());
        }
        
        // 简化的格式化，实际应该使用专门的格式化器
        Ok(format!("{}", statements[0]))
    }
    
    /// 从SQL中提取表名（简化实现）
    fn extract_table_from_sql(&self, sql: &str, keyword: &str) -> Option<String> {
        let sql_upper = sql.to_uppercase();
        if let Some(start) = sql_upper.find(keyword) {
            let after_keyword = &sql_upper[start + keyword.len()..];
            let words: Vec<&str> = after_keyword.split_whitespace().collect();
            if !words.is_empty() {
                return Some(words[0].to_lowercase());
            }
        }
        None
    }

    /// 提取查询中的字面量
    pub fn extract_literals(&self, sql: &str) -> Result<Vec<String>> {
        let statements = Parser::parse_sql(&self.dialect, sql)
            .map_err(|e| Error::validation(format!("SQL parsing failed: {}", e)))?;
        
        let mut literals = Vec::new();
        
        // 简化实现，实际需要遍历AST提取所有字面量
        for statement in statements {
            let statement_str = format!("{:?}", statement);
            // 这里应该有更复杂的字面量提取逻辑
            if statement_str.contains("String(") {
                literals.push("string_literal".to_string());
            }
            if statement_str.contains("Number(") {
                literals.push("number_literal".to_string());
            }
        }
        
        Ok(literals)
    }
}

impl Default for SqlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = SqlParser::new();
        assert!(parser.validate("SELECT 1").is_ok());
    }

    #[test]
    fn test_simple_select() {
        let parser = SqlParser::new();
        let result = parser.parse("SELECT * FROM users").unwrap();
        
        assert_eq!(result.query_type, QueryType::Select);
        assert_eq!(result.tables.len(), 1);
        assert_eq!(result.tables[0], "users");
        assert!(result.is_readonly);
    }

    #[test]
    fn test_join_query() {
        let parser = SqlParser::new();
        let result = parser.parse("SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id").unwrap();
        
        assert_eq!(result.query_type, QueryType::Select);
        assert_eq!(result.tables.len(), 2);
        assert!(result.tables.contains(&"users".to_string()));
        assert!(result.tables.contains(&"orders".to_string()));
        assert!(result.is_multi_table());
    }

    #[test]
    fn test_insert_query() {
        let parser = SqlParser::new();
        let result = parser.parse("INSERT INTO users (name, email) VALUES ('John', 'john@example.com')").unwrap();
        
        assert_eq!(result.query_type, QueryType::Insert);
        assert_eq!(result.tables.len(), 1);
        assert_eq!(result.tables[0], "users");
        assert!(!result.is_readonly);
    }

    #[test]
    fn test_invalid_sql() {
        let parser = SqlParser::new();
        let result = parser.parse("INVALID SQL STATEMENT");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation() {
        let parser = SqlParser::new();
        assert!(parser.validate("SELECT * FROM users").is_ok());
        assert!(parser.validate("INVALID SQL").is_err());
    }

    #[test]
    fn test_format() {
        let parser = SqlParser::new();
        let formatted = parser.format("select   *   from   users").unwrap();
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_extract_literals() {
        let parser = SqlParser::new();
        let literals = parser.extract_literals("SELECT * FROM users WHERE name = 'John' AND age = 25").unwrap();
        assert!(!literals.is_empty());
    }
}
