//! Financial-specific type definitions

use crate::definition::{TypeDefinition, TypeKind, PrimitiveType, FieldDefinition, TypeConstraint};
use fdc_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;

/// 金融类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FinancialType {
    /// 价格类型
    Price,
    /// 成交量类型
    Volume,
    /// 货币类型
    Currency,
    /// 期权合约类型
    OptionContract,
    /// 期货合约类型
    FutureContract,
    /// 债券类型
    Bond,
    /// 股票类型
    Stock,
    /// 外汇对类型
    CurrencyPair,
    /// 利率类型
    InterestRate,
    /// 波动率类型
    Volatility,
}

/// 价格类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceType {
    /// 精度（小数位数）
    pub precision: u8,
    /// 最小价格变动单位
    pub tick_size: Decimal,
    /// 货币代码
    pub currency: String,
    /// 价格范围
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
}

impl PriceType {
    /// 创建新的价格类型
    pub fn new(precision: u8, tick_size: Decimal, currency: String) -> Self {
        Self {
            precision,
            tick_size,
            currency,
            min_price: None,
            max_price: None,
        }
    }
    
    /// 设置价格范围
    pub fn with_range(mut self, min_price: Decimal, max_price: Decimal) -> Self {
        self.min_price = Some(min_price);
        self.max_price = Some(max_price);
        self
    }
    
    /// 转换为类型定义
    pub fn to_type_definition(&self, name: String) -> TypeDefinition {
        let mut type_def = TypeDefinition::new(name, TypeKind::Custom("price".to_string()))
            .with_description(format!("Price type with {} decimal places in {}", self.precision, self.currency));
        
        // 添加精度约束
        type_def.set_attribute("precision".to_string(), self.precision.to_string());
        type_def.set_attribute("tick_size".to_string(), self.tick_size.to_string());
        type_def.set_attribute("currency".to_string(), self.currency.clone());
        
        // 添加范围约束
        if let Some(min) = self.min_price {
            type_def.add_constraint(TypeConstraint::MinValue(min.to_f64().unwrap_or(0.0)));
        }
        if let Some(max) = self.max_price {
            type_def.add_constraint(TypeConstraint::MaxValue(max.to_f64().unwrap_or(f64::MAX)));
        }
        
        type_def
    }
}

/// 成交量类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeType {
    /// 最小成交量单位
    pub lot_size: u64,
    /// 最大成交量
    pub max_volume: Option<u64>,
    /// 成交量单位名称
    pub unit: String,
}

impl VolumeType {
    /// 创建新的成交量类型
    pub fn new(lot_size: u64, unit: String) -> Self {
        Self {
            lot_size,
            max_volume: None,
            unit,
        }
    }
    
    /// 设置最大成交量
    pub fn with_max_volume(mut self, max_volume: u64) -> Self {
        self.max_volume = Some(max_volume);
        self
    }
    
    /// 转换为类型定义
    pub fn to_type_definition(&self, name: String) -> TypeDefinition {
        let mut type_def = TypeDefinition::new(name, TypeKind::Custom("volume".to_string()))
            .with_description(format!("Volume type with lot size {} {}", self.lot_size, self.unit));
        
        type_def.set_attribute("lot_size".to_string(), self.lot_size.to_string());
        type_def.set_attribute("unit".to_string(), self.unit.clone());
        
        // 添加最小值约束（必须是lot_size的倍数）
        type_def.add_constraint(TypeConstraint::MinValue(self.lot_size as f64));
        
        if let Some(max) = self.max_volume {
            type_def.add_constraint(TypeConstraint::MaxValue(max as f64));
        }
        
        type_def
    }
}

/// 货币类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyType {
    /// 货币代码（ISO 4217）
    pub code: String,
    /// 货币名称
    pub name: String,
    /// 小数位数
    pub decimal_places: u8,
    /// 符号
    pub symbol: Option<String>,
}

impl CurrencyType {
    /// 创建新的货币类型
    pub fn new(code: String, name: String, decimal_places: u8) -> Self {
        Self {
            code,
            name,
            decimal_places,
            symbol: None,
        }
    }
    
    /// 设置货币符号
    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = Some(symbol);
        self
    }
    
    /// 转换为类型定义
    pub fn to_type_definition(&self, name: String) -> TypeDefinition {
        let mut type_def = TypeDefinition::new(name, TypeKind::Custom("currency".to_string()))
            .with_description(format!("Currency type for {} ({})", self.name, self.code));
        
        type_def.set_attribute("code".to_string(), self.code.clone());
        type_def.set_attribute("name".to_string(), self.name.clone());
        type_def.set_attribute("decimal_places".to_string(), self.decimal_places.to_string());
        
        if let Some(ref symbol) = self.symbol {
            type_def.set_attribute("symbol".to_string(), symbol.clone());
        }
        
        // 添加货币代码格式约束
        type_def.add_constraint(TypeConstraint::Pattern("^[A-Z]{3}$".to_string()));
        
        type_def
    }
}

/// 期权合约类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContractType {
    /// 标的资产
    pub underlying: String,
    /// 期权类型（看涨/看跌）
    pub option_type: OptionType,
    /// 行权价格类型
    pub strike_price_type: PriceType,
    /// 到期日格式
    pub expiry_format: String,
    /// 合约乘数
    pub multiplier: u32,
}

/// 期权类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionType {
    Call,
    Put,
}

impl OptionContractType {
    /// 创建新的期权合约类型
    pub fn new(
        underlying: String,
        option_type: OptionType,
        strike_price_type: PriceType,
        multiplier: u32,
    ) -> Self {
        Self {
            underlying,
            option_type,
            strike_price_type,
            expiry_format: "YYYY-MM-DD".to_string(),
            multiplier,
        }
    }
    
    /// 转换为类型定义
    pub fn to_type_definition(&self, name: String) -> TypeDefinition {
        let mut type_def = TypeDefinition::new(name, TypeKind::Struct)
            .with_description(format!("{:?} option contract for {}", self.option_type, self.underlying));
        
        // 添加字段
        let underlying_field = FieldDefinition::new(
            "underlying".to_string(),
            TypeDefinition::new("string".to_string(), TypeKind::Primitive(PrimitiveType::String)),
        ).with_description("Underlying asset symbol".to_string());
        
        let option_type_field = FieldDefinition::new(
            "option_type".to_string(),
            TypeDefinition::new("string".to_string(), TypeKind::Primitive(PrimitiveType::String)),
        ).with_description("Option type (Call/Put)".to_string());
        
        let strike_price_field = FieldDefinition::new(
            "strike_price".to_string(),
            self.strike_price_type.to_type_definition("strike_price_type".to_string()),
        ).with_description("Strike price".to_string());
        
        let expiry_field = FieldDefinition::new(
            "expiry_date".to_string(),
            TypeDefinition::new("timestamp".to_string(), TypeKind::Primitive(PrimitiveType::Timestamp)),
        ).with_description("Expiry date".to_string());
        
        let multiplier_field = FieldDefinition::new(
            "multiplier".to_string(),
            TypeDefinition::new("u32".to_string(), TypeKind::Primitive(PrimitiveType::U32)),
        ).with_description("Contract multiplier".to_string());
        
        type_def.add_field(underlying_field);
        type_def.add_field(option_type_field);
        type_def.add_field(strike_price_field);
        type_def.add_field(expiry_field);
        type_def.add_field(multiplier_field);
        
        // 添加属性
        type_def.set_attribute("underlying".to_string(), self.underlying.clone());
        type_def.set_attribute("option_type".to_string(), format!("{:?}", self.option_type));
        type_def.set_attribute("multiplier".to_string(), self.multiplier.to_string());
        
        type_def
    }
}

/// 期货合约类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureContractType {
    /// 标的资产
    pub underlying: String,
    /// 合约月份格式
    pub contract_month_format: String,
    /// 价格类型
    pub price_type: PriceType,
    /// 合约乘数
    pub multiplier: u32,
    /// 最后交易日规则
    pub last_trading_day_rule: String,
}

impl FutureContractType {
    /// 创建新的期货合约类型
    pub fn new(
        underlying: String,
        price_type: PriceType,
        multiplier: u32,
    ) -> Self {
        Self {
            underlying,
            contract_month_format: "YYYYMM".to_string(),
            price_type,
            multiplier,
            last_trading_day_rule: "Third Friday of contract month".to_string(),
        }
    }
    
    /// 转换为类型定义
    pub fn to_type_definition(&self, name: String) -> TypeDefinition {
        let mut type_def = TypeDefinition::new(name, TypeKind::Struct)
            .with_description(format!("Future contract for {}", self.underlying));
        
        // 添加字段
        let underlying_field = FieldDefinition::new(
            "underlying".to_string(),
            TypeDefinition::new("string".to_string(), TypeKind::Primitive(PrimitiveType::String)),
        ).with_description("Underlying asset symbol".to_string());
        
        let contract_month_field = FieldDefinition::new(
            "contract_month".to_string(),
            TypeDefinition::new("string".to_string(), TypeKind::Primitive(PrimitiveType::String)),
        ).with_description("Contract month".to_string());
        
        let price_field = FieldDefinition::new(
            "price".to_string(),
            self.price_type.to_type_definition("future_price_type".to_string()),
        ).with_description("Future price".to_string());
        
        let multiplier_field = FieldDefinition::new(
            "multiplier".to_string(),
            TypeDefinition::new("u32".to_string(), TypeKind::Primitive(PrimitiveType::U32)),
        ).with_description("Contract multiplier".to_string());
        
        type_def.add_field(underlying_field);
        type_def.add_field(contract_month_field);
        type_def.add_field(price_field);
        type_def.add_field(multiplier_field);
        
        // 添加属性
        type_def.set_attribute("underlying".to_string(), self.underlying.clone());
        type_def.set_attribute("contract_month_format".to_string(), self.contract_month_format.clone());
        type_def.set_attribute("multiplier".to_string(), self.multiplier.to_string());
        type_def.set_attribute("last_trading_day_rule".to_string(), self.last_trading_day_rule.clone());
        
        type_def
    }
}

/// 创建常用的金融类型
pub fn create_common_financial_types() -> Vec<TypeDefinition> {
    let mut types = Vec::new();
    
    // USD价格类型
    let usd_price = PriceType::new(
        2,
        Decimal::new(1, 2), // 0.01
        "USD".to_string(),
    ).with_range(
        Decimal::new(1, 4), // 0.0001
        Decimal::new(1000000, 0), // 1,000,000
    );
    types.push(usd_price.to_type_definition("USDPrice".to_string()));
    
    // 股票成交量类型
    let stock_volume = VolumeType::new(
        1, // 最小1股
        "shares".to_string(),
    ).with_max_volume(1_000_000_000); // 10亿股
    types.push(stock_volume.to_type_definition("StockVolume".to_string()));
    
    // 美元货币类型
    let usd_currency = CurrencyType::new(
        "USD".to_string(),
        "US Dollar".to_string(),
        2,
    ).with_symbol("$".to_string());
    types.push(usd_currency.to_type_definition("USD".to_string()));
    
    // 股票期权类型
    let stock_option = OptionContractType::new(
        "STOCK".to_string(),
        OptionType::Call,
        usd_price,
        100, // 标准合约乘数
    );
    types.push(stock_option.to_type_definition("StockOption".to_string()));
    
    types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_type() {
        let price_type = PriceType::new(
            2,
            Decimal::new(1, 2),
            "USD".to_string(),
        );
        
        let type_def = price_type.to_type_definition("TestPrice".to_string());
        assert_eq!(type_def.name, "TestPrice");
        assert_eq!(type_def.attributes.get("precision"), Some(&"2".to_string()));
        assert_eq!(type_def.attributes.get("currency"), Some(&"USD".to_string()));
    }

    #[test]
    fn test_volume_type() {
        let volume_type = VolumeType::new(100, "shares".to_string())
            .with_max_volume(1_000_000);
        
        let type_def = volume_type.to_type_definition("TestVolume".to_string());
        assert_eq!(type_def.name, "TestVolume");
        assert_eq!(type_def.attributes.get("lot_size"), Some(&"100".to_string()));
        assert_eq!(type_def.attributes.get("unit"), Some(&"shares".to_string()));
    }

    #[test]
    fn test_currency_type() {
        let currency_type = CurrencyType::new(
            "EUR".to_string(),
            "Euro".to_string(),
            2,
        ).with_symbol("€".to_string());
        
        let type_def = currency_type.to_type_definition("TestCurrency".to_string());
        assert_eq!(type_def.name, "TestCurrency");
        assert_eq!(type_def.attributes.get("code"), Some(&"EUR".to_string()));
        assert_eq!(type_def.attributes.get("symbol"), Some(&"€".to_string()));
    }

    #[test]
    fn test_option_contract_type() {
        let price_type = PriceType::new(2, Decimal::new(1, 2), "USD".to_string());
        let option_type = OptionContractType::new(
            "AAPL".to_string(),
            OptionType::Call,
            price_type,
            100,
        );
        
        let type_def = option_type.to_type_definition("AAPLOption".to_string());
        assert_eq!(type_def.name, "AAPLOption");
        assert_eq!(type_def.fields.len(), 5);
        assert_eq!(type_def.attributes.get("underlying"), Some(&"AAPL".to_string()));
    }

    #[test]
    fn test_common_financial_types() {
        let types = create_common_financial_types();
        assert!(!types.is_empty());
        
        let type_names: Vec<&str> = types.iter().map(|t| t.name.as_str()).collect();
        assert!(type_names.contains(&"USDPrice"));
        assert!(type_names.contains(&"StockVolume"));
        assert!(type_names.contains(&"USD"));
        assert!(type_names.contains(&"StockOption"));
    }
}
