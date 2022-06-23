use std::sync::Arc;
use std::sync::atomic::AtomicI64;
use uuid::Uuid;
use crate::traits::Scheduled;

pub struct Task {
    pub id: Uuid,
    pub expiration: i64,
    pub handler: Arc<dyn Fn() + Sync + Send + 'static>,
    pub scheduled: Arc<dyn Scheduled>
}

impl Task {
    pub fn update(&mut self) -> bool{
        return if let Some(expiration) = self.scheduled.next() {
            self.expiration = expiration;
            true
        } else {
            false
        }
    }
}

impl Clone for Task {
    fn clone(&self) -> Self {
        Self{
            id: self.id,
            expiration: self.expiration,
            handler: self.handler.clone(),
            scheduled: self.scheduled.clone(),
        }
    }
}