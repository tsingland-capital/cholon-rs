use std::collections::{BinaryHeap, BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use chrono::{Local, TimeZone};
use log::{info, trace};
use crate::buckets::Bucket;

pub struct DelayQueueInner {
    queue: Vec<Bucket>,
}

pub struct DelayQueue {
    inner: Arc<Mutex<DelayQueueInner>>
}

impl DelayQueue {
    pub fn new() -> Self{
        Self{
            inner: Arc::new(
                Mutex::new(
                    DelayQueueInner {
                        queue: vec![],
                    }
                )
            )
        }
    }

    pub fn push(&self, bucket: Bucket) {
        let mut queue = self.inner.lock().unwrap();
        queue.queue.push(bucket);
        queue.queue.sort();
    }

    pub fn peek_and_shift(&self, expiration: i64) -> Vec<Bucket> {
        let mut buckets = vec![];
        let mut max_count = 0;
        let mut queue = self.inner.lock().unwrap();
        for idx in 0..queue.queue.len() {
            let bucket = queue.queue.get(idx).unwrap();
            if bucket.get_expiration() <= expiration {
                trace!("检查到过期Bucket: {}", Local.timestamp_millis(bucket.get_expiration()).format("%Y-%m-%d %H:%M:%S"));
                buckets.push(bucket.clone());
                max_count += 1;
            } else {
                break
            }
        };
        // 移除开头过期元素
        for idx in 0..max_count{
            queue.queue.remove(0);
        }
        buckets
    }


}

impl Clone for DelayQueue {
    fn clone(&self) -> Self {
        Self{
            inner: self.inner.clone()
        }
    }
}