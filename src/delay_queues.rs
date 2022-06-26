use std::collections::{BinaryHeap, BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use crate::buckets::Bucket;

pub struct DelayQueueInner {
    priority_queue: BTreeSet<Bucket>,
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
                        priority_queue: BTreeSet::new()
                    }
                )
            )
        }
    }

    pub fn push(&self, bucket: Bucket) {
        let mut queue = self.inner.lock().unwrap();
        queue.priority_queue.insert(bucket);
    }

    pub fn peek_and_shift(&self, expiration: i64) -> Option<Bucket> {

        // let mut expired_buckets = vec![];

        // {
        //     let queue = self.inner.lock().unwrap();
        //     for bucket in queue.priority_queue {
        //         if bucket.get_expiration() <= expiration{
        //             expired_buckets.push(bucket);
        //         }
        //     }
        //     if let Some(bucket) = queue.priority_queue.iter().next(){
        //         Some(bucket.clone())
        //     } else { None }
        // }
        if let Some(bucket) = {
            let queue = self.inner.lock().unwrap();
            if let Some(bucket) = queue.priority_queue.iter().next(){
                Some(bucket.clone())
            } else { None }
        } {
            if bucket.get_expiration() <= expiration {
                {
                    let mut queue = self.inner.lock().unwrap();
                    queue.priority_queue.remove(&bucket);
                }
                Some(bucket)
            } else {
                None
            }
        } else { None }
    }


}

impl Clone for DelayQueue {
    fn clone(&self) -> Self {
        Self{
            inner: self.inner.clone()
        }
    }
}