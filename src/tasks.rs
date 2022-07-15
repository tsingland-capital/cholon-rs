use std::future::Future;
use std::sync::Arc;
use uuid::Uuid;
use crate::common::{AsyncFn, SyncFn};
use super::containers::Container;
use super::routines::Routine;

#[derive(Clone)]
pub enum Executable{
    Future(Arc<AsyncFn>),
    Block(Arc<SyncFn>),
}

#[derive(Clone)]
pub struct Task {
    pub id: Uuid,
    pub expiration: i64,
    pub executable: Executable,
    pub routine: Arc<dyn Routine>
}

impl Task {
    /// 更新任务过期时间以重新加入到Bucket
    pub fn update(&mut self) -> bool{
        return if let Some(expiration) = self.routine.next() {
            self.expiration = expiration;
            true
        } else {
            false
        }
    }
}
