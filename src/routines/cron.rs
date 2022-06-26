use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use chrono::{TimeZone, Utc};
use cron::Schedule;
use super::Routine;

pub struct Cron {
    max_count: Option<i64>,
    count: AtomicI64,
    updated_at: AtomicI64,
    cron_schedule: Arc<Schedule>,
}

impl Cron {
    pub fn new(created_at: i64, expression: &str, max_count: Option<i64>) -> Self{
        Self{
            max_count,
            count: AtomicI64::new(0),
            updated_at: AtomicI64::new(created_at),
            cron_schedule: Arc::new(Schedule::from_str(expression).unwrap()),
        }
    }
}

impl Routine for Cron {
    fn next(&self) -> Option<i64> {
        // todo 针对不同时区配置
        if let Some(datetime) = self.cron_schedule.after(&Utc.timestamp_millis(self.updated_at.load(Ordering::Relaxed))).next(){
            let next_expiration = datetime.timestamp_millis();
            self.updated_at.store(next_expiration, Ordering::Relaxed);
            Some(next_expiration)
        } else {
            None
        }
    }
}
