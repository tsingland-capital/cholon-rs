use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use chrono::{NaiveDateTime, TimeZone, Utc};
use cron::Schedule;
use uuid::Uuid;

/// 时间源
pub trait TimeSource: Send + Sync + 'static {
    /// 获取当前时间戳
    fn now(&self) -> i64;
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
}

pub trait Unique {
    // 任务ID
    fn id(&self) -> Uuid;

    fn cancel(&self);
    // 任务是否被取消, 当从Bucket提取出时检查是否已被取消，未被取消则继续执行
    fn cancelled(&self) -> bool;
}
// 任务
pub trait Scheduled: Sync + Send + 'static{
    // 获取下一次运行时间，执行完成后会获取下一次运行时间，若存在则重新添加到时间轮种
    fn next(&self) -> Option<i64>;
}

pub struct Once {}

impl Once {
    pub fn new() -> Self{
        Self{}
    }
}
impl Scheduled for Once {
    fn next(&self) -> Option<i64> {
        None
    }
}

pub struct Timeout {
    created_at: i64,
    count: AtomicI64,
    timeout: i64,
}

impl Timeout {
    pub fn new(created_at: i64, timeout: i64) -> Self{
        Self{
            created_at,
            count: AtomicI64::new(0),
            timeout,
        }
    }
}

impl Scheduled for Timeout {
    fn next(&self) -> Option<i64> {
        // let count = self.count.load(Ordering::Relaxed);
        let count = self.count.fetch_add(1, Ordering::Relaxed);
        Some(self.created_at + count * self.timeout)
    }
}

pub struct Cron {
    updated_at: AtomicI64,
    cron_schedule: Arc<Schedule>,
}

impl Cron {
    pub fn new(created_at: i64, expression: &str) -> Self{
        Self{
            updated_at: AtomicI64::new(created_at),
            cron_schedule: Arc::new(Schedule::from_str(expression).unwrap()),
        }
    }
}
impl Scheduled for Cron {
    fn next(&self) -> Option<i64> {
        if let Some(datetime) = self.cron_schedule.after(&Utc.timestamp_millis(self.updated_at.load(Ordering::Relaxed))).next(){
            let next_expiration = datetime.timestamp_millis();
            self.updated_at.store(next_expiration, Ordering::Relaxed);
            Some(next_expiration)
        } else {
            None
        }
    }
}