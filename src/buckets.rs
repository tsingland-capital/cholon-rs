use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicI64, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};
use chrono::{TimeZone, Local};
use log::{info, trace};
use crate::tasks::Task;

pub struct BucketInner {
    expiration: AtomicI64,
    tasks: Mutex<Vec<Task>>,
}

#[derive(Clone)]
pub struct Bucket {
    inner: Arc<BucketInner>
}


impl Eq for Bucket {}

impl PartialEq<Self> for Bucket {
    fn eq(&self, other: &Self) -> bool {
        self.get_expiration().eq(&other.get_expiration())
    }
}

impl PartialOrd<Self> for Bucket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {

        self.get_expiration().partial_cmp(&other.get_expiration())
    }
}

impl Ord for Bucket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_expiration().cmp(&other.get_expiration())
    }
}


impl Bucket {
    pub fn new() -> Self{
        Self{
            inner: Arc::new(
                BucketInner{
                    expiration: AtomicI64::new(-1),
                    tasks: Mutex::new(vec![])
                }
            )
        }
    }

    pub fn set_expiration(&self, expiration: i64) -> bool {
        return if self.get_expiration() != expiration {
            self.inner.expiration.store(expiration, AtomicOrdering::Relaxed);
            true
        } else {
            false
        }
    }
    pub fn get_expiration(&self) -> i64{
        self.inner.expiration.load(AtomicOrdering::Relaxed)
    }

    pub fn add(&self, task: Task) {
        let mut tasks = self.inner.tasks.lock().unwrap();
        tasks.push(task);
        trace!("当前Bucket包含的任务个数{}, 过期时间: {}", tasks.len(), Local.timestamp_millis(self.get_expiration()).format("%Y-%m-%d %H:%M:%S"));
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        let mut tasks = self.inner.tasks.lock().unwrap();
        let expired = tasks.to_vec();
        tasks.clear();
        return expired
    }
}