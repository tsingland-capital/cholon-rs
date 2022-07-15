use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::future::Future;
use futures::future::{Shared, BoxFuture};
use futures::prelude::*;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;
use chrono::{TimeZone, Utc};
use tokio::time::{Instant, Interval, interval_at, sleep};
use cholon::*;
use async_trait::async_trait;
use cron::Schedule;
use futures::FutureExt;
use tokio::runtime::Runtime;
use tokio::time;
use cholon::containers::Container;

fn main() {
    let schedule = Schedule::from_str("3 * * * * *").unwrap();
    for next_time in schedule.after(&Utc::now()){
        println!("{}", next_time.format("%Y-%m-%d %H:%M:%S"));
    }
}

fn test_container() {
    let f = || async move {
        // time::sleep(Duration::from_secs(2)).await;
        println!("测试");
    }.boxed();


    let rtm = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    rtm.schedule_async(Arc::new(|| async move {
        sleep(Duration::from_secs(2)).await;
        println!("测试schedule_async");
    }.boxed()), None);

    rtm.schedule_block(Arc::new(|| {
        println!("测试schedule_block");
    }), None);

    // thread::sleep(Duration::from_secs(100));
    thread::sleep(Duration::from_secs(100));

}
