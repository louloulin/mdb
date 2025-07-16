//! Time windowing for stream processing

use crate::config::WindowingConfig;
use std::collections::VecDeque;
use std::time::Duration;

/// 时间窗口
pub struct TimeWindow<T> {
    /// 窗口数据
    data: VecDeque<(chrono::DateTime<chrono::Utc>, T)>,
    /// 窗口大小
    window_size: Duration,
    /// 最大容量
    max_capacity: usize,
}

impl<T> TimeWindow<T> {
    /// 创建新的时间窗口
    pub fn new(window_size: Duration, max_capacity: usize) -> Self {
        Self {
            data: VecDeque::new(),
            window_size,
            max_capacity,
        }
    }
    
    /// 添加数据点
    pub fn add(&mut self, timestamp: chrono::DateTime<chrono::Utc>, value: T) {
        // 移除过期数据
        let cutoff = timestamp - self.window_size;
        while let Some((ts, _)) = self.data.front() {
            if *ts < cutoff {
                self.data.pop_front();
            } else {
                break;
            }
        }
        
        // 添加新数据
        self.data.push_back((timestamp, value));
        
        // 限制容量
        while self.data.len() > self.max_capacity {
            self.data.pop_front();
        }
    }
    
    /// 获取窗口大小
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// 获取窗口数据的引用
    pub fn data(&self) -> &VecDeque<(chrono::DateTime<chrono::Utc>, T)> {
        &self.data
    }
}

/// 窗口管理器
pub struct WindowManager<T> {
    config: WindowingConfig,
    windows: Vec<TimeWindow<T>>,
}

impl<T> WindowManager<T> {
    /// 创建新的窗口管理器
    pub fn new(config: WindowingConfig) -> Self {
        Self {
            config,
            windows: Vec::new(),
        }
    }
    
    /// 创建新窗口
    pub fn create_window(&mut self) -> usize {
        let window = TimeWindow::new(self.config.default_window_size, 1000);
        self.windows.push(window);
        self.windows.len() - 1
    }
    
    /// 获取窗口
    pub fn get_window(&mut self, index: usize) -> Option<&mut TimeWindow<T>> {
        self.windows.get_mut(index)
    }
    
    /// 获取窗口数量
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_window() {
        let window_size = Duration::from_secs(60);
        let mut window = TimeWindow::new(window_size, 100);
        
        let now = chrono::Utc::now();
        window.add(now, "data1");
        window.add(now + Duration::from_secs(30), "data2");
        
        assert_eq!(window.size(), 2);
        assert!(!window.is_empty());
        
        // 添加过期数据
        window.add(now + Duration::from_secs(120), "data3");
        assert_eq!(window.size(), 1); // 前两个数据点应该被移除
    }

    #[test]
    fn test_window_manager() {
        let config = WindowingConfig::default();
        let mut manager: WindowManager<String> = WindowManager::new(config);
        
        let window_id = manager.create_window();
        assert_eq!(window_id, 0);
        assert_eq!(manager.window_count(), 1);
        
        let window = manager.get_window(window_id).unwrap();
        window.add(chrono::Utc::now(), "test".to_string());
        assert_eq!(window.size(), 1);
    }
}
