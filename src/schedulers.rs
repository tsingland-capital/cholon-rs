use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use rayon::{ThreadPool, ThreadPoolBuilder};
use uuid::Uuid;
use crate::utils::truncate;
use crate::delay_queues::DelayQueue;
use crate::errors::Error;
use crate::tasks::Task;
use crate::timing_wheels::TimingWheel;
use crate::traits::{Cron, Timeout, TimeSource, SystemTimeSource, Once, Scheduled};

pub struct SchedulerBuilder {
    time_source: Option<Arc<dyn TimeSource>>,
    thread_pool: Option<Arc<ThreadPool>>,
    tick: i64,
    size: usize,
}

impl SchedulerBuilder {
    pub fn new() -> Self{
        Self{
            time_source: None,
            thread_pool: None,
            tick: 1000,
            size: 100,
        }
    }
}

impl SchedulerBuilder {
    pub fn with_time_source<T>(mut self, time_source: T) -> Self where T: 'static + TimeSource{
        self.time_source = Some(Arc::new(time_source));
        self
    }
    pub fn with_thread_pool(mut self, thread_pool: Arc<ThreadPool>) -> Self{
        self.thread_pool = Some(thread_pool);
        self
    }
    pub fn with_tick(mut self, tick: i64) -> Self{
        self.tick = tick;
        self
    }
    pub fn with_size(mut self, size: usize) -> Self{
        self.size = size;
        self
    }
}

impl Default for SchedulerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SchedulerBuilder{
    pub fn build(self) -> Result<Scheduler, Error> {
        if self.size <= 1 {
            return Err(Error::InvalidParam("timing wheel size must be greater than 1"));
        }
        if self.tick <= 1 {
            return Err(Error::InvalidParam("timing wheel tick must be greater than 1"));
        }
        let time_source = match self.time_source {
            Some(time_source) => time_source,
            None => Arc::new(SystemTimeSource::new()),
        };


        let queue = DelayQueue::new();
        let now = time_source.now();
        Ok(Scheduler{
            inner: Arc::new(SchedulerInner {
                quit: AtomicBool::new(false),
                tick_ms: self.tick,
                timing_wheel: TimingWheel::new(self.tick, self.size as i64, now, queue.clone()),
                queue,
                time_source,
                thread_pool: match self.thread_pool {
                    Some(thread_pool) => thread_pool,
                    None => Arc::new(ThreadPoolBuilder::new().
                        num_threads(4).
                        thread_name(|idx| format!("cholon_task_pool_{}", idx)).
                        build().
                        unwrap()),
                },

            })
        })
    }
}

struct SchedulerInner {
    quit: AtomicBool,
    tick_ms: i64,
    timing_wheel: TimingWheel,
    queue: DelayQueue,
    time_source: Arc<dyn TimeSource>,
    thread_pool: Arc<ThreadPool>,
}

pub struct Scheduler {
    inner: Arc<SchedulerInner>,
}

impl Scheduler {
    pub fn new<T>(
        tick_ms: i64, wheel_size: i64, time_source: T, thread_pool: Arc<ThreadPool>,
    ) -> Self where T: TimeSource {
        let queue = DelayQueue::new();
        let now = time_source.now();
        Self{
            inner: Arc::new(SchedulerInner {
                quit: AtomicBool::new(false),
                tick_ms,
                timing_wheel: TimingWheel::new(tick_ms, wheel_size, now, queue.clone()),
                queue,
                time_source: Arc::new(time_source),
                thread_pool,
            })
        }
    }

    pub fn schedule(&self, task: Task) -> bool {
        self.inner.timing_wheel.schedule(task)
    }

    pub fn schedule_once<F>(&self, timeout: i64, f: F) -> bool where F: Fn() + Sync + Send + 'static {
        let now = self.inner.time_source.now();
        let task = Task{
            id: Uuid::new_v4(),
            expiration: now + timeout,
            handler: Arc::new(f),
            scheduled: Arc::new(Once::new())
        };
        self.inner.timing_wheel.schedule(task)
    }

    pub fn schedule_cron<F>(&self, expression: &str, f: F) -> bool where F: Fn() + Sync + Send + 'static {
        let now = self.inner.time_source.now();
        let cron_schedule = Cron::new(now, expression);
        let task = Task{
            id: Uuid::new_v4(),
            expiration: cron_schedule.next().unwrap(),
            handler: Arc::new(f),
            scheduled: Arc::new(cron_schedule),
        };
        self.inner.timing_wheel.schedule(task)
    }

    pub fn schedule_timeout<F>(&self, timeout: i64, f: F) -> bool where F: Fn() + Sync + Send + 'static {
        let now = self.inner.time_source.now();
        let timeout = Timeout::new(now, timeout);
        let task = Task{
            id: Uuid::new_v4(),
            expiration: timeout.next().unwrap(),
            handler: Arc::new(f),
            scheduled: Arc::new(timeout)
        };
        self.inner.timing_wheel.schedule(task)
    }

    pub fn start(&self) {
        let scheduler = self.inner.clone();
        self.inner.thread_pool.spawn(move ||{
            loop {
                if scheduler.quit.load(Ordering::Relaxed){
                    return;
                }
                let now = truncate(scheduler.time_source.now(), scheduler.tick_ms);
                if let Some(bucket) = scheduler.queue.peek_and_shift(now){
                    for mut task in bucket.get_tasks() {
                        let handler = task.handler.clone();
                        scheduler.thread_pool.spawn(move || {
                            (handler)()
                        });
                        if task.update(){
                            scheduler.timing_wheel.schedule(task);
                        }
                    }
                }
            }
        })
    }

    pub fn stop(&self) {
        self.inner.quit.store(true, Ordering::Relaxed)
    }
}
