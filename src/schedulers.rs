use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use futures::future::BoxFuture;
use uuid::Uuid;
use crate::containers::Container;
use crate::routines::{Cron, Routine, Timeout};
use crate::tasks::{Executable, Task};
use crate::common::{TimeSource, SyncFn, AsyncFn};
use crate::timing_wheels::TimingWheel;
use crate::delay_queues::DelayQueue;
use crate::utils::truncate;
use futures::prelude::*;

pub struct SchedulerInner {
    quit: AtomicBool,
    container: Arc<dyn Container>,
    time_source: Arc<dyn TimeSource>,
    queue: DelayQueue,
    tick_ms: i64,
    timing_wheel: TimingWheel,
}

impl SchedulerInner {
    pub fn heartbeat(&self) {
        if self.quit.load(Ordering::Relaxed){
            return;
        }
        let now = truncate(self.time_source.now(), self.tick_ms);
        self.timing_wheel.advance_clock(now);
        if let Some(bucket) = self.queue.peek_and_shift(now){
            // println!("调度器执行检查, {}", now);
            for mut task in bucket.get_tasks() {
                match &task.executable {
                    Executable::Future(async_fn) => {
                        self.container.schedule_async(async_fn.clone())
                    }
                    Executable::Block(sync_fn) => {
                        self.container.schedule_block(sync_fn.clone())
                    }
                }
                if task.update(){
                    println!("重新插入任务, {}", task.expiration);
                    self.timing_wheel.schedule(task);
                }
            }
        }
    }
}
pub struct Scheduler {
    inner: Arc<SchedulerInner>,
}

impl Scheduler{
    pub fn new<Ctr, TS>(
        tick_ms: i64, wheel_size: i64, time_source: TS, container: Ctr,
    ) -> Self where Ctr: Container, TS: TimeSource {
        let queue = DelayQueue::new();
        let now = time_source.now();
        Self {
            inner: Arc::new(SchedulerInner {
                quit: AtomicBool::new(false),
                container: Arc::new(container),
                tick_ms,
                timing_wheel: TimingWheel::new(tick_ms, wheel_size, now, queue.clone()),
                time_source: Arc::new(time_source),
                queue
            })
        }
    }

    pub fn schedule(&self, executable: Executable, routine: Arc<dyn Routine>) -> bool {
        if let Some(expiration) = routine.next() {
            let task = Task {
                id: Uuid::new_v4(),
                expiration,
                executable,
                routine,
            };
            self.inner.timing_wheel.schedule(task)
        } else {
            false
        }
    }

    pub fn schedule_block_cron<F>(&self, expression: &str, f: F) -> bool where F: Fn() -> () + Send + Sync + 'static {
        let now = self.inner.time_source.now();
        let cron_schedule = Cron::new(now, expression, None);
        self.schedule(Executable::Block(Arc::new(f)), Arc::new(cron_schedule))
    }

    pub fn schedule_async_cron<F>(&self, expression: &str, f: F) -> bool where F: Fn() -> (BoxFuture<'static, ()>) + Send + Sync + 'static {
        let now = self.inner.time_source.now();
        let cron_schedule = Cron::new(now, expression, None);
        self.schedule(Executable::Future(Arc::new(f)), Arc::new(cron_schedule))
    }

    pub fn schedule_block_timeout<F>(&self, timeout: i64, f: F) -> bool where F: Fn() -> () + Send + Sync + 'static {
        let now = self.inner.time_source.now();
        let timeout = Timeout::new(now, timeout, None);
        self.schedule(Executable::Block(Arc::new(f)), Arc::new(timeout))
    }

    pub fn schedule_async_timeout<F>(&self, timeout: i64, f: F) -> bool where F: Fn() -> (BoxFuture<'static, ()>) + Send + Sync + 'static {
        let now = self.inner.time_source.now();
        let timeout = Timeout::new(now, timeout, None);
        self.schedule(Executable::Future(Arc::new(f)), Arc::new(timeout))
    }
    pub fn start(&self) {

        let scheduler = self.inner.clone();
        self.inner.container.run_forever(
            Instant::now(),
            Duration::from_secs(1),
            Arc::new(move || {
                let scheduler = scheduler.clone();
                async move {
                    scheduler.heartbeat();
                }.boxed()
            }
        ));
    }

    pub fn stop(&self) {
        self.inner.quit.store(true, Ordering::Relaxed)
    }

    pub fn heartbeat(&self) {
        self.inner.heartbeat()
    }
}