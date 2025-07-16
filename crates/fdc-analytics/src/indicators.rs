//! Technical indicators calculation

use crate::{config::IndicatorsConfig, models::SignalType};
use fdc_core::error::Result;

/// 技术指标计算器
pub struct TechnicalIndicators {
    config: IndicatorsConfig,
}

impl TechnicalIndicators {
    /// 创建新的技术指标计算器
    pub fn new(config: IndicatorsConfig) -> Self {
        Self { config }
    }
    
    /// 计算简单移动平均
    pub fn calculate_sma(&self, prices: &[f64], window: usize) -> Result<Vec<f64>> {
        if prices.len() < window || window == 0 {
            return Ok(Vec::new());
        }

        let mut sma = Vec::new();
        for i in 0..prices.len() {
            if i + 1 >= window {
                let start_idx = i + 1 - window;
                let sum: f64 = prices[start_idx..=i].iter().sum();
                sma.push(sum / window as f64);
            }
        }

        Ok(sma)
    }
    
    /// 计算RSI
    pub fn calculate_rsi(&self, prices: &[f64]) -> Result<Vec<f64>> {
        if prices.len() < self.config.rsi_period + 1 {
            return Ok(Vec::new());
        }
        
        let mut rsi = Vec::new();
        let mut gains = Vec::new();
        let mut losses = Vec::new();
        
        // 计算价格变化
        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }
        
        // 计算RSI
        for i in self.config.rsi_period - 1..gains.len() {
            let avg_gain: f64 = gains[i - self.config.rsi_period + 1..=i].iter().sum::<f64>() / self.config.rsi_period as f64;
            let avg_loss: f64 = losses[i - self.config.rsi_period + 1..=i].iter().sum::<f64>() / self.config.rsi_period as f64;
            
            let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
            let rsi_value = 100.0 - (100.0 / (1.0 + rs));
            rsi.push(rsi_value);
        }
        
        Ok(rsi)
    }
    
    /// 生成交易信号
    pub fn generate_signal(&self, indicator_name: &str, value: f64) -> SignalType {
        match indicator_name {
            "rsi" => {
                if value > 70.0 {
                    SignalType::Sell
                } else if value < 30.0 {
                    SignalType::Buy
                } else {
                    SignalType::Hold
                }
            }
            _ => SignalType::Neutral,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_calculation() {
        let config = IndicatorsConfig::default();
        let indicators = TechnicalIndicators::new(config);
        
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sma = indicators.calculate_sma(&prices, 3).unwrap();
        
        assert_eq!(sma.len(), 3);
        assert_eq!(sma[0], 2.0); // (1+2+3)/3
        assert_eq!(sma[1], 3.0); // (2+3+4)/3
        assert_eq!(sma[2], 4.0); // (3+4+5)/3
    }

    #[test]
    fn test_signal_generation() {
        let config = IndicatorsConfig::default();
        let indicators = TechnicalIndicators::new(config);
        
        assert!(matches!(indicators.generate_signal("rsi", 80.0), SignalType::Sell));
        assert!(matches!(indicators.generate_signal("rsi", 20.0), SignalType::Buy));
        assert!(matches!(indicators.generate_signal("rsi", 50.0), SignalType::Hold));
    }
}
