use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI64, Ordering};
use chrono::{Local, TimeZone, Utc};
use cron::{OwnedScheduleIterator, Schedule, ScheduleIterator};
use log::trace;
use super::Routine;

pub struct Cron {
    max_count: Option<i64>,
    created_at: i64,
    count: AtomicI64,
    // updated_at: AtomicI64,
    schedule_iter: Arc<Mutex<OwnedScheduleIterator<Utc>>>,
}

impl Cron {
    pub fn new(created_at: i64, expression: &str, max_count: Option<i64>) -> Self{
        let schedule = Schedule::from_str(expression).unwrap();
        let iter = Arc::new(Mutex::new(schedule.after_owned(Utc.timestamp_millis(created_at))));
        Self{
            max_count,
            created_at,
            count: AtomicI64::new(0),
            // updated_at: AtomicI64::new(created_at),
            schedule_iter: iter,
        }
    }
}

impl Routine for Cron {
    fn next(&self) -> Option<i64> {
        // todo 针对不同时区配置
        let mut schedule = self.schedule_iter.lock().unwrap();
        if let Some(datetime) = schedule.next(){
            let next_expiration = datetime.timestamp_millis();
            // self.updated_at.store(next_expiration, Ordering::Relaxed);
            trace!("创建时间: {}, 拉起新调用时间: {}", Local.timestamp_millis(self.created_at).format("%Y-%m-%d %H:%M:%S"), Local.timestamp_millis(next_expiration).format("%Y-%m-%d %H:%M:%S"));
            Some(next_expiration)
        } else {
            None
        }
    }
}
