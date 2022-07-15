pub mod schedulers;
pub mod containers;
pub mod buckets;
pub mod common;
pub mod utils;
pub mod errors;
pub mod tasks;
pub mod routines;
pub mod timing_wheels;
pub mod delay_queues;
pub mod prelude;

pub use schedulers::Scheduler;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
