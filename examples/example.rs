use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use chrono::{Duration, Local, NaiveDateTime, TimeZone, Utc};
use futures::FutureExt;
use tokio::time;
use cholon::common::{SystemTimeSource, TimeSource};
use cholon::schedulers::Scheduler;

#[derive(Clone)]
struct SimulateTimeSource{
    time: Arc<AtomicI64>,
}

impl SimulateTimeSource {
    pub fn new(start: i64) -> Self{
        Self{
            time: Arc::new(AtomicI64::new(start))
        }
    }

    pub fn update_time(&self){
        let old = self.time.fetch_add(1000, Ordering::Relaxed);
        println!("时间更新: {}", Local.timestamp_millis(old+1000).format("%Y-%m-%d %H:%M:%S"));
    }
}

impl TimeSource for SimulateTimeSource {
    fn now(&self) -> i64 {
        self.time.load(Ordering::Relaxed)
    }
}
fn main() {

    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let time_source = SimulateTimeSource::new(1641009600000);
    let scheduler = Scheduler::new(
        1000,
        100,
        time_source.clone(),
        runtime
    );
    // scheduler.start();
    println!("开始, {}",  Local.timestamp_millis(time_source.now()).format("%Y-%m-%d %H:%M:%S"));
    std::thread::sleep(Duration::seconds(2).to_std().unwrap());
    let once_job = || {
        println!("单次任务正在执行：{}", Utc::now().format("%Y-%m-%d %H:%M:%S"))
    };
    let timeout_job = {
        let async_ts = time_source.clone();
        move || {
            let ts = async_ts.clone();
            async move {
                // time::sleep(Duration::seconds(1).to_std().unwrap()).await;
                println!("延迟任务正在执行：{}", Local.timestamp_millis(ts.now()).format("%Y-%m-%d %H:%M:%S"))
            }.boxed()
        }
    };
    let cron_job ={
        let ts = time_source.clone();
        move || {
            println!("定时任务正在执行：{}", Local.timestamp_millis(ts.now()).format("%Y-%m-%d %H:%M:%S"))
        }
    };
    // scheduler.schedule_once(1000 * 5, once_job);
    scheduler.schedule_async_timeout(1000 * 30, timeout_job);
    scheduler.schedule_block_cron("10 * * * * *", cron_job);
    println!("调度, {}",  Local.timestamp_millis(time_source.now()).format("%Y-%m-%d %H:%M:%S"));
    for i in 0..1000 {
        if i % 2 == 0 {
            time_source.update_time();
        }
        // std::thread::sleep(Duration::milliseconds(100).to_std().unwrap());
        // println!("调度, {}", Utc::now());
        scheduler.heartbeat();
    }
    // tokio::spawn(async move {
    //     let mut interval = time::interval(time::Duration::from_secs(1));
    //     loop {
    //         interval.tick().await;
    //         scheduler.heartbeat();
    //     }
    // });
    // scheduler.stop();
}