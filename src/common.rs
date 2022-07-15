use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use chrono::{TimeZone, Utc};
use cron::Schedule;
use futures::future::BoxFuture;

pub type AsyncFn = dyn Fn() -> (BoxFuture<'static, ()>) + Send + Sync + 'static;
pub type SyncFn = dyn Fn() -> () + Send + Sync + 'static;

/// 时间源
pub trait TimeSource: Send + Sync + 'static {
    /// 获取当前时间戳
    fn now(&self) -> i64;
    fn update_now(&self, val: i64) -> bool;
}


#[derive(Default, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SystemTimeSource {}

impl SystemTimeSource {
    pub fn new() -> Self{
        Self{}
    }
}

impl TimeSource for SystemTimeSource {
    fn now(&self) -> i64 {
        Utc::now().timestamp_millis()
    }
    fn update_now(&self, val: i64) -> bool {
        true
    }
}

#[derive(Clone)]
pub struct SimulateTimeSource{
    time: Arc<AtomicI64>,
}

impl SimulateTimeSource {
    pub fn new(start: i64) -> Self{
        Self{
            time: Arc::new(AtomicI64::new(start)),
        }
    }
}

impl TimeSource for SimulateTimeSource {
    fn now(&self) -> i64 {
        self.time.load(Ordering::Relaxed)
    }

    fn update_now(&self, val: i64) -> bool{
        let last = self.time.swap(val, Ordering::Relaxed);
        if last == 0 {
            false
        } else {
            last != val
        }
    }
}

pub trait TimeSourceProvider{
    fn time_source(&self) -> SimulateTimeSource;
}
