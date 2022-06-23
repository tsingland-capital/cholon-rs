use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI64, Ordering as AtomicOrdering};
use crate::tasks::Task;

struct BucketInner{
    expiration: AtomicI64,
    tasks: Mutex<Vec<Task>>,
}

pub struct Bucket {
    inner: Arc<BucketInner>
}

impl Eq for Bucket {}

impl PartialEq<Self> for Bucket {
    fn eq(&self, other: &Self) -> bool {
        self.get_expiration() == other.get_expiration()
    }
}

impl PartialOrd<Self> for Bucket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.get_expiration() > other.get_expiration(){
            Some(Ordering::Greater)
        } else if self.get_expiration() < other.get_expiration() {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Ord for Bucket {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.get_expiration() > other.get_expiration(){
            Ordering::Greater
        } else if self.get_expiration() < other.get_expiration() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl Clone for Bucket {
    fn clone(&self) -> Self {
        Self{
            inner: self.inner.clone()
        }
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
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        let mut tasks = self.inner.tasks.lock().unwrap();
        let expired = tasks.to_vec();
        tasks.clear();
        return expired
    }
}