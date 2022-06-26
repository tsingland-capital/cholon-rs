mod cron;
mod timeout;

pub use self::cron::Cron;
pub use self::timeout::Timeout;

pub trait Routine: Sync + Send + 'static{
    // 获取下一次运行时间，执行完成后会获取下一次运行时间，若存在则重新添加到时间轮种
    fn next(&self) -> Option<i64>;
}
