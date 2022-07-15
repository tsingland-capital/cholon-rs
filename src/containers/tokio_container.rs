use std::future::Future;
use futures::prelude::*;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use crossbeam::sync::WaitGroup;
use futures::future::BoxFuture;
use tokio::runtime::Runtime;
use tokio::time;
use tokio::time::{Interval, interval_at};
use crate::common::{AsyncFn, SyncFn};
use crate::containers::Container;

impl Container for Runtime {
    fn schedule_async(&self, executable: Arc<AsyncFn>, wg: Option<WaitGroup>) {
        self.spawn(async move {
            (executable)().await;
            if let Some(v) = wg {
                drop(v)
            }
        });
    }

    fn schedule_block(&self, executable: Arc<SyncFn>, wg: Option<WaitGroup>) {
        self.spawn_blocking(move || {
            (executable)();
            if let Some(v) = wg {
                drop(v)
            }
            // drop(wg)
        });
    }

    fn run_forever(&self, start: Instant, duration: Duration, executable: Arc<AsyncFn>) {
        self.spawn(async move {
            let mut clock = TokioClock::new(time::Instant::from_std(start), duration);
            loop {
                clock.tick().await;
                (executable)().await;
            }
        });
    }
}


#[derive(Debug)]
struct TokioClock {
    inner: Interval,
}

impl TokioClock {
    pub fn new(start: time::Instant, period: Duration) -> Self {
        let inner = interval_at(start, period);
        TokioClock { inner }
    }

    pub async fn tick(&mut self) {
        self.inner.tick().await;
    }
}

