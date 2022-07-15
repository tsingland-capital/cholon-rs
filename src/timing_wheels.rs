use std::sync::{Arc, Mutex};
use chrono::{Local, TimeZone};
use log::{debug, trace};
use super::buckets::Bucket;
use super::delay_queues::DelayQueue;
use super::tasks::Task;
use super::utils::truncate;

struct TimingWheelInner{
    pub(crate) tick_ms: i64,
    pub(crate) wheel_size: i64,
    pub(crate) total_interval: i64,
    pub(crate) current_time: i64,
    pub(crate) buckets: Vec<Bucket>,
    pub(crate) queue: DelayQueue,
    pub(crate) overflow_wheel: Option<TimingWheel>,
}

impl TimingWheelInner {
    fn add_overflow_wheel(&mut self) {
        if self.overflow_wheel.is_none(){
            // println!("溢出时间轮创建, 精度: {}, 创建时间: {}",
            //          self.tick_ms * self.wheel_size,
            //          Local.timestamp_millis(self.current_time).format("%Y-%m-%d %H:%M:%S")
            // );
            self.overflow_wheel = Some(
                TimingWheel::new(
                    self.tick_ms * self.wheel_size,
                    self.wheel_size,
                    self.current_time,
                    self.queue.clone(),
                )
            );
            // let overflow_wheel_guard = self.overflow_wheel.as_ref().unwrap();
            // let overflow_wheel = overflow_wheel_guard.inner.lock().unwrap();
            // println!("溢出时间轮添加成功: {}", overflow_wheel.tick_ms)
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
        let start_time = truncate(start_time, tick_ms);

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
        // println!("任务[{}] 添加到队列，下一次执行时间为: {}", task.id, Local.timestamp_millis(task.expiration).format("%Y-%m-%d %H:%M:%S"));

        return if task.expiration < wheel.current_time + wheel.tick_ms {
            trace!("任务[{}] 添加到队列失败，下一次执行时间为: {}，已超时", task.id, Local.timestamp_millis(task.expiration).format("%Y-%m-%d %H:%M:%S"));
            false
        } else if task.expiration < wheel.current_time + wheel.total_interval {
            trace!("任务[{}] 被添加到队列，下一次执行时间为: {}", task.id, Local.timestamp_millis(task.expiration).format("%Y-%m-%d %H:%M:%S"));
            let virtual_id = task.expiration / wheel.tick_ms;
            let bucket = &wheel.buckets[(virtual_id % wheel.wheel_size) as usize];
            bucket.add(task);
            if bucket.set_expiration(virtual_id * wheel.tick_ms) {
                trace!("更新对应Bucket到期时间为: {}", Local.timestamp_millis(virtual_id * wheel.tick_ms).format("%Y-%m-%d %H:%M:%S"));
                wheel.queue.push(bucket.clone());
            }
            true
        } else {
            trace!("任务[{}] 溢出队列，下一次执行时间为: {}", task.id, Local.timestamp_millis(task.expiration).format("%Y-%m-%d %H:%M:%S"));
            wheel.add_overflow_wheel();
            wheel.overflow_wheel.as_ref().unwrap().schedule(task)
        }
    }

    pub fn advance_clock(&self, time: i64) {
        let mut wheel = self.inner.lock().unwrap();
        if time >= wheel.current_time + wheel.tick_ms {
            // println!("时间轮更新[精度={}] 当前时间: {}, 更新时间: {}",
            //          wheel.tick_ms,
            //          Local.timestamp_millis(wheel.current_time).
            //              format("%Y-%m-%d %H:%M:%S"),
            //          Local.timestamp_millis(time).
            //              format("%Y-%m-%d %H:%M:%S"),
            // );

            wheel.current_time = truncate(time, wheel.tick_ms);
            if let Some(overflow_wheel) = &wheel.overflow_wheel {
                overflow_wheel.advance_clock(time);
            }
        }
    }
}