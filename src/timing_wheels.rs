use std::sync::{Arc, Mutex};
use crate::buckets::Bucket;
use crate::delay_queues::DelayQueue;
use crate::tasks::Task;
use crate::utils::truncate;

struct TimingWheelInner{
    tick_ms: i64,
    wheel_size: i64,
    total_interval: i64,
    current_time: i64,
    buckets: Vec<Bucket>,
    queue: DelayQueue,
    overflow_wheel: Option<TimingWheel>,
}

impl TimingWheelInner {
    fn add_overflow_wheel(&mut self) {
        if self.overflow_wheel.is_none(){
            self.overflow_wheel = Some(
                TimingWheel::new(
                    self.tick_ms * self.wheel_size,
                    self.wheel_size,
                    self.current_time,
                    self.queue.clone(),
                )
            )
        }
    }
}

pub struct TimingWheel{
    inner: Arc<Mutex<TimingWheelInner>>
}

impl TimingWheel {
    pub fn new(tick_ms: i64, wheel_size: i64, start_time: i64, queue: DelayQueue) -> Self{

        // 创建Bucket列表
        let mut buckets = Vec::with_capacity(wheel_size as usize);
        // 初始化Bucket
        for idx in 0..wheel_size {
            buckets.insert(idx as usize, Bucket::new());
        }

        Self{
            inner: Arc::new(Mutex::new(
                TimingWheelInner{
                    tick_ms,
                    wheel_size,
                    total_interval: tick_ms * wheel_size,
                    current_time: start_time,
                    buckets,
                    queue,
                    overflow_wheel: None
                }
            ))
        }
    }

    /// 将任务加入到时间轮中
    pub fn schedule(&self, task: Task) -> bool{
        let mut wheel = self.inner.lock().unwrap();
        return if task.expiration < wheel.current_time + wheel.tick_ms {
            false
        } else if task.expiration < wheel.current_time + wheel.total_interval {
            let virtual_id = task.expiration / wheel.tick_ms;
            let bucket = &wheel.buckets[(virtual_id % wheel.wheel_size) as usize];
            bucket.add(task);
            if bucket.set_expiration(virtual_id * wheel.tick_ms) {
                wheel.queue.push(bucket.clone());
            }
            true
        } else {
            wheel.add_overflow_wheel();
            wheel.overflow_wheel.as_ref().unwrap().schedule(task)
        }
    }

    pub fn advance_clock(&self, time: i64) {
        let mut wheel = self.inner.lock().unwrap();
        if time >= wheel.current_time + wheel.tick_ms {
            wheel.current_time = truncate(time, wheel.tick_ms);
            if let Some(overflow_wheel) = &wheel.overflow_wheel {
                overflow_wheel.advance_clock(time);
            }
        }
    }
}