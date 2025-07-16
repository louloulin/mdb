//! Metrics collection for Financial Data Center

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// 指标收集器
#[derive(Debug)]
pub struct Metrics {
    counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    gauges: Arc<RwLock<HashMap<String, AtomicU64>>>,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 增加计数器
    pub fn increment_counter(&self, name: &str, value: u64) {
        let counters = self.counters.read();
        if let Some(counter) = counters.get(name) {
            counter.fetch_add(value, Ordering::Relaxed);
        } else {
            drop(counters);
            let mut counters = self.counters.write();
            let counter = counters.entry(name.to_string()).or_insert_with(|| AtomicU64::new(0));
            counter.fetch_add(value, Ordering::Relaxed);
        }
    }
    
    /// 设置仪表值
    pub fn set_gauge(&self, name: &str, value: u64) {
        let gauges = self.gauges.read();
        if let Some(gauge) = gauges.get(name) {
            gauge.store(value, Ordering::Relaxed);
        } else {
            drop(gauges);
            let mut gauges = self.gauges.write();
            let gauge = gauges.entry(name.to_string()).or_insert_with(|| AtomicU64::new(0));
            gauge.store(value, Ordering::Relaxed);
        }
    }
    
    /// 记录直方图值
    pub fn record_histogram(&self, name: &str, value: f64) {
        let histograms = self.histograms.read();
        if let Some(histogram) = histograms.get(name) {
            histogram.record(value);
        } else {
            drop(histograms);
            let mut histograms = self.histograms.write();
            let histogram = histograms.entry(name.to_string()).or_insert_with(Histogram::new);
            histogram.record(value);
        }
    }
    
    /// 获取计数器值
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        let counters = self.counters.read();
        counters.get(name).map(|c| c.load(Ordering::Relaxed))
    }
    
    /// 获取仪表值
    pub fn get_gauge(&self, name: &str) -> Option<u64> {
        let gauges = self.gauges.read();
        gauges.get(name).map(|g| g.load(Ordering::Relaxed))
    }
    
    /// 获取直方图统计
    pub fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        let histograms = self.histograms.read();
        histograms.get(name).map(|h| h.stats())
    }
    
    /// 获取所有指标快照
    pub fn snapshot(&self) -> MetricsSnapshot {
        let counters = self.counters.read();
        let gauges = self.gauges.read();
        let histograms = self.histograms.read();
        
        let counter_values: HashMap<String, u64> = counters
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect();
        
        let gauge_values: HashMap<String, u64> = gauges
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect();
        
        let histogram_stats: HashMap<String, HistogramStats> = histograms
            .iter()
            .map(|(k, v)| (k.clone(), v.stats()))
            .collect();
        
        MetricsSnapshot {
            counters: counter_values,
            gauges: gauge_values,
            histograms: histogram_stats,
        }
    }
    
    /// 重置所有指标
    pub fn reset(&self) {
        let mut counters = self.counters.write();
        let mut gauges = self.gauges.write();
        let mut histograms = self.histograms.write();
        
        counters.clear();
        gauges.clear();
        histograms.clear();
    }
}

/// 直方图
#[derive(Debug)]
pub struct Histogram {
    values: RwLock<Vec<f64>>,
    count: AtomicU64,
    sum: RwLock<f64>,
}

impl Histogram {
    /// 创建新的直方图
    pub fn new() -> Self {
        Self {
            values: RwLock::new(Vec::new()),
            count: AtomicU64::new(0),
            sum: RwLock::new(0.0),
        }
    }
    
    /// 记录值
    pub fn record(&self, value: f64) {
        let mut values = self.values.write();
        let mut sum = self.sum.write();
        
        values.push(value);
        *sum += value;
        self.count.fetch_add(1, Ordering::Relaxed);
        
        // 保持最近1000个值
        if values.len() > 1000 {
            values.remove(0);
        }
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> HistogramStats {
        let values = self.values.read();
        let sum = *self.sum.read();
        let count = self.count.load(Ordering::Relaxed);
        
        if values.is_empty() {
            return HistogramStats::default();
        }
        
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min = sorted_values[0];
        let max = sorted_values[sorted_values.len() - 1];
        let mean = sum / count as f64;
        
        let p50 = percentile(&sorted_values, 0.5);
        let p95 = percentile(&sorted_values, 0.95);
        let p99 = percentile(&sorted_values, 0.99);
        
        HistogramStats {
            count,
            sum,
            min,
            max,
            mean,
            p50,
            p95,
            p99,
        }
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// 计算百分位数
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let index = (p * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

/// 直方图统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramStats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

impl Default for HistogramStats {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

/// 指标快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, u64>,
    pub histograms: HashMap<String, HistogramStats>,
}

/// 常用指标名称
pub mod metric_names {
    pub const INGESTION_MESSAGES_TOTAL: &str = "ingestion_messages_total";
    pub const INGESTION_BYTES_TOTAL: &str = "ingestion_bytes_total";
    pub const INGESTION_LATENCY: &str = "ingestion_latency_seconds";
    pub const INGESTION_ERRORS_TOTAL: &str = "ingestion_errors_total";
    
    pub const QUERY_REQUESTS_TOTAL: &str = "query_requests_total";
    pub const QUERY_LATENCY: &str = "query_latency_seconds";
    pub const QUERY_ERRORS_TOTAL: &str = "query_errors_total";
    pub const QUERY_CACHE_HITS_TOTAL: &str = "query_cache_hits_total";
    pub const QUERY_CACHE_MISSES_TOTAL: &str = "query_cache_misses_total";
    
    pub const STORAGE_WRITES_TOTAL: &str = "storage_writes_total";
    pub const STORAGE_READS_TOTAL: &str = "storage_reads_total";
    pub const STORAGE_WRITE_LATENCY: &str = "storage_write_latency_seconds";
    pub const STORAGE_READ_LATENCY: &str = "storage_read_latency_seconds";
    pub const STORAGE_ERRORS_TOTAL: &str = "storage_errors_total";
    
    pub const WASM_PLUGIN_CALLS_TOTAL: &str = "wasm_plugin_calls_total";
    pub const WASM_PLUGIN_LATENCY: &str = "wasm_plugin_latency_seconds";
    pub const WASM_PLUGIN_ERRORS_TOTAL: &str = "wasm_plugin_errors_total";
    pub const WASM_MEMORY_USAGE: &str = "wasm_memory_usage_bytes";
    
    pub const MEMORY_USAGE: &str = "memory_usage_bytes";
    pub const CPU_USAGE: &str = "cpu_usage_percent";
    pub const DISK_USAGE: &str = "disk_usage_bytes";
    pub const NETWORK_BYTES_SENT: &str = "network_bytes_sent_total";
    pub const NETWORK_BYTES_RECEIVED: &str = "network_bytes_received_total";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_counter() {
        let metrics = Metrics::new();
        
        metrics.increment_counter("test_counter", 1);
        metrics.increment_counter("test_counter", 2);
        
        assert_eq!(metrics.get_counter("test_counter"), Some(3));
    }

    #[test]
    fn test_metrics_gauge() {
        let metrics = Metrics::new();
        
        metrics.set_gauge("test_gauge", 42);
        assert_eq!(metrics.get_gauge("test_gauge"), Some(42));
        
        metrics.set_gauge("test_gauge", 100);
        assert_eq!(metrics.get_gauge("test_gauge"), Some(100));
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new();
        
        histogram.record(1.0);
        histogram.record(2.0);
        histogram.record(3.0);
        histogram.record(4.0);
        histogram.record(5.0);
        
        let stats = histogram.stats();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.mean, 3.0);
    }

    #[test]
    fn test_metrics_snapshot() {
        let metrics = Metrics::new();
        
        metrics.increment_counter("counter1", 10);
        metrics.set_gauge("gauge1", 20);
        metrics.record_histogram("hist1", 1.5);
        
        let snapshot = metrics.snapshot();
        
        assert_eq!(snapshot.counters.get("counter1"), Some(&10));
        assert_eq!(snapshot.gauges.get("gauge1"), Some(&20));
        assert!(snapshot.histograms.contains_key("hist1"));
    }
}
