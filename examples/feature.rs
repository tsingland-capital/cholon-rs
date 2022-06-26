use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::future::Future;
use futures::future::{Shared, BoxFuture};
use futures::prelude::*;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;
use chrono::Utc;
use tokio::time::{Instant, Interval, interval_at, sleep};
use cholon::v1::*;
use cholon::v1::containers::Container;
use async_trait::async_trait;
use futures::FutureExt;
use tokio::runtime::Runtime;
use tokio::time;

fn main() {
    let f = || async move {
        // time::sleep(Duration::from_secs(2)).await;
        println!("测试");
    }.boxed();


    let rtm = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    rtm.schedule_async(Arc::new(|| async move {
        sleep(Duration::from_secs(2)).await;
        println!("测试schedule_async");
    }.boxed()));

    rtm.schedule_block(Arc::new(|| {
        println!("测试schedule_block");
    }));

    // thread::sleep(Duration::from_secs(100));
    thread::sleep(Duration::from_secs(100));

}
