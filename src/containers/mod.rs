//! 运行时容器，承载cholon挂载的异步运行时容器
//!
//! 需要支持异步运行、同步运行接口

use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::tasks::Task;
use async_trait::async_trait;
use futures::future::BoxFuture;
use crate::common::{AsyncFn, SyncFn};

mod tokio_container;


pub trait Container: Send + Sync + 'static {
    fn schedule_async(&self, executable: Arc<AsyncFn>);
    fn schedule_block(&self, executable: Arc<SyncFn>);
    fn run_forever(&self, start: Instant, duration: Duration, executable: Arc<AsyncFn>);
}

