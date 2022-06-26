use std::sync::atomic::{AtomicI64, Ordering};
use super::Routine;

pub struct Timeout {
    max_count: Option<i64>,
    created_at: i64,
    count: AtomicI64,
    timeout: i64,
}

impl Timeout {
    pub fn new(created_at: i64, timeout: i64, max_count: Option<i64>) -> Self{
        Self{
            max_count,
            created_at,
            count: AtomicI64::new(1),
            timeout,
        }
    }
}

impl Routine for Timeout {
    fn next(&self) -> Option<i64> {
        // let count = self.count.load(Ordering::Relaxed);
        let count = self.count.fetch_add(1, Ordering::Relaxed);
        Some(self.created_at + count * self.timeout)
    }
}
