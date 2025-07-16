//! Time utilities for Financial Data Center

use chrono::{DateTime, Utc, Timelike, TimeZone};
use crate::types::TimestampNs;
use crate::error::{Error, Result};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// 时间工具类
pub struct TimeUtils;

impl TimeUtils {
    /// 获取当前纳秒时间戳
    pub fn now_nanos() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_nanos() as i64
    }
    
    /// 获取当前微秒时间戳
    pub fn now_micros() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_micros() as i64
    }
    
    /// 获取当前毫秒时间戳
    pub fn now_millis() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as i64
    }
    
    /// 获取当前秒时间戳
    pub fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs() as i64
    }
    
    /// 从字符串解析时间戳
    pub fn parse_timestamp(s: &str) -> Result<TimestampNs> {
        use chrono::NaiveDateTime;

        // 尝试多种格式
        let formats = [
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
        ];

        // 首先尝试带时区的格式
        for format in &formats {
            if let Ok(dt) = DateTime::parse_from_str(s, format) {
                return Ok(TimestampNs::from_nanos(dt.timestamp_nanos_opt().unwrap_or(0)));
            }
        }

        // 然后尝试不带时区的格式
        let naive_formats = [
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S",
        ];

        for format in &naive_formats {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s, format) {
                let dt = Utc.from_utc_datetime(&naive_dt);
                return Ok(TimestampNs::from_nanos(dt.timestamp_nanos_opt().unwrap_or(0)));
            }
        }

        // 尝试解析为纯数字（纳秒时间戳）
        if let Ok(nanos) = s.parse::<i64>() {
            return Ok(TimestampNs::from_nanos(nanos));
        }

        Err(Error::parse(format!("Unable to parse timestamp: {}", s)))
    }
    
    /// 格式化时间戳为字符串
    pub fn format_timestamp(ts: TimestampNs, format: &str) -> Result<String> {
        if let Some(dt) = ts.to_datetime() {
            Ok(dt.format(format).to_string())
        } else {
            Err(Error::parse(format!("Invalid timestamp: {}", ts.as_nanos())))
        }
    }
    
    /// 获取时间戳的日期部分（YYYY-MM-DD）
    pub fn get_date(ts: TimestampNs) -> Result<String> {
        Self::format_timestamp(ts, "%Y-%m-%d")
    }
    
    /// 获取时间戳的时间部分（HH:MM:SS）
    pub fn get_time(ts: TimestampNs) -> Result<String> {
        Self::format_timestamp(ts, "%H:%M:%S")
    }
    
    /// 获取时间戳的小时
    pub fn get_hour(ts: TimestampNs) -> Result<u32> {
        if let Some(dt) = ts.to_datetime() {
            Ok(dt.hour())
        } else {
            Err(Error::parse(format!("Invalid timestamp: {}", ts.as_nanos())))
        }
    }
    
    /// 获取时间戳的分钟
    pub fn get_minute(ts: TimestampNs) -> Result<u32> {
        if let Some(dt) = ts.to_datetime() {
            Ok(dt.minute())
        } else {
            Err(Error::parse(format!("Invalid timestamp: {}", ts.as_nanos())))
        }
    }
    
    /// 获取时间戳的秒
    pub fn get_second(ts: TimestampNs) -> Result<u32> {
        if let Some(dt) = ts.to_datetime() {
            Ok(dt.second())
        } else {
            Err(Error::parse(format!("Invalid timestamp: {}", ts.as_nanos())))
        }
    }
    
    /// 时间戳向下取整到指定间隔
    pub fn floor_to_interval(ts: TimestampNs, interval_nanos: i64) -> TimestampNs {
        let nanos = ts.as_nanos();
        let floored = (nanos / interval_nanos) * interval_nanos;
        TimestampNs::from_nanos(floored)
    }
    
    /// 时间戳向上取整到指定间隔
    pub fn ceil_to_interval(ts: TimestampNs, interval_nanos: i64) -> TimestampNs {
        let nanos = ts.as_nanos();
        let ceiled = ((nanos + interval_nanos - 1) / interval_nanos) * interval_nanos;
        TimestampNs::from_nanos(ceiled)
    }
    
    /// 检查时间戳是否在指定范围内
    pub fn is_in_range(ts: TimestampNs, start: TimestampNs, end: TimestampNs) -> bool {
        ts >= start && ts <= end
    }
    
    /// 计算两个时间戳的差值（纳秒）
    pub fn diff_nanos(ts1: TimestampNs, ts2: TimestampNs) -> i64 {
        ts1.as_nanos() - ts2.as_nanos()
    }
    
    /// 添加纳秒到时间戳
    pub fn add_nanos(ts: TimestampNs, nanos: i64) -> TimestampNs {
        TimestampNs::from_nanos(ts.as_nanos() + nanos)
    }
    
    /// 减去纳秒从时间戳
    pub fn sub_nanos(ts: TimestampNs, nanos: i64) -> TimestampNs {
        TimestampNs::from_nanos(ts.as_nanos() - nanos)
    }
}

/// 时间间隔常量（纳秒）
pub mod intervals {
    pub const NANOSECOND: i64 = 1;
    pub const MICROSECOND: i64 = 1_000;
    pub const MILLISECOND: i64 = 1_000_000;
    pub const SECOND: i64 = 1_000_000_000;
    pub const MINUTE: i64 = 60 * SECOND;
    pub const HOUR: i64 = 60 * MINUTE;
    pub const DAY: i64 = 24 * HOUR;
    pub const WEEK: i64 = 7 * DAY;
}

/// 时间范围
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeRange {
    pub start: TimestampNs,
    pub end: TimestampNs,
}

impl TimeRange {
    /// 创建新的时间范围
    pub fn new(start: TimestampNs, end: TimestampNs) -> Result<Self> {
        if start > end {
            return Err(Error::invalid_argument("Start time must be before end time"));
        }
        Ok(Self { start, end })
    }
    
    /// 检查时间戳是否在范围内
    pub fn contains(&self, ts: TimestampNs) -> bool {
        TimeUtils::is_in_range(ts, self.start, self.end)
    }
    
    /// 获取范围的持续时间（纳秒）
    pub fn duration_nanos(&self) -> i64 {
        TimeUtils::diff_nanos(self.end, self.start)
    }
    
    /// 检查两个时间范围是否重叠
    pub fn overlaps(&self, other: &TimeRange) -> bool {
        self.start <= other.end && self.end >= other.start
    }
    
    /// 获取两个时间范围的交集
    pub fn intersection(&self, other: &TimeRange) -> Option<TimeRange> {
        if !self.overlaps(other) {
            return None;
        }
        
        let start = std::cmp::max(self.start, other.start);
        let end = std::cmp::min(self.end, other.end);
        
        TimeRange::new(start, end).ok()
    }
    
    /// 获取两个时间范围的并集
    pub fn union(&self, other: &TimeRange) -> TimeRange {
        let start = std::cmp::min(self.start, other.start);
        let end = std::cmp::max(self.end, other.end);
        
        TimeRange::new(start, end).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_utils() {
        let now = TimeUtils::now_nanos();
        assert!(now > 0);
        
        let ts = TimestampNs::from_nanos(now);
        assert_eq!(ts.as_nanos(), now);
    }

    #[test]
    fn test_parse_timestamp() {
        let ts_str = "2023-01-01 12:00:00";
        let ts = TimeUtils::parse_timestamp(ts_str).unwrap();
        assert!(ts.as_nanos() > 0);
    }

    #[test]
    fn test_time_range() {
        let start = TimestampNs::from_nanos(1000);
        let end = TimestampNs::from_nanos(2000);
        let range = TimeRange::new(start, end).unwrap();
        
        assert!(range.contains(TimestampNs::from_nanos(1500)));
        assert!(!range.contains(TimestampNs::from_nanos(500)));
        assert!(!range.contains(TimestampNs::from_nanos(2500)));
    }

    #[test]
    fn test_time_range_overlap() {
        let range1 = TimeRange::new(
            TimestampNs::from_nanos(1000),
            TimestampNs::from_nanos(2000)
        ).unwrap();
        
        let range2 = TimeRange::new(
            TimestampNs::from_nanos(1500),
            TimestampNs::from_nanos(2500)
        ).unwrap();
        
        assert!(range1.overlaps(&range2));
        
        let intersection = range1.intersection(&range2).unwrap();
        assert_eq!(intersection.start.as_nanos(), 1500);
        assert_eq!(intersection.end.as_nanos(), 2000);
    }

    #[test]
    fn test_floor_ceil_interval() {
        let ts = TimestampNs::from_nanos(1234567890);
        let interval = intervals::SECOND; // 1秒 = 1,000,000,000纳秒
        
        let floored = TimeUtils::floor_to_interval(ts, interval);
        let ceiled = TimeUtils::ceil_to_interval(ts, interval);
        
        assert_eq!(floored.as_nanos(), 1000000000);
        assert_eq!(ceiled.as_nanos(), 2000000000);
    }
}
