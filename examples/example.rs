use cholon::prelude::*;

fn main() {
    let scheduler = SchedulerBuilder::new().with_size(100).with_tick(1000).build().unwrap();
    scheduler.start();
    println!("开始, {}", Utc::now());
    std::thread::sleep(Duration::seconds(2).to_std().unwrap());
    let once_job = || {
        println!("单次任务正在执行：{}", Utc::now().format("%Y-%m-%d %H:%M:%S"))
    };
    let timeout_job = || {
        println!("延迟任务正在执行：{}", Utc::now().format("%Y-%m-%d %H:%M:%S"))
    };
    let cron_job = || {
        println!("定时任务正在执行：{}", Utc::now().format("%Y-%m-%d %H:%M:%S"))
    };
    scheduler.schedule_once(1000 * 5, once_job);
    scheduler.schedule_timeout(1000 * 3, timeout_job);
    scheduler.schedule_cron("0 * * * * *", cron_job);
    println!("调度, {}", Utc::now());
    std::thread::sleep(Duration::seconds(150).to_std().unwrap());
    scheduler.stop();
}