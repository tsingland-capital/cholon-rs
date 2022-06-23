
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub enum Error{
    InvalidParam(&'static str),
    ScheduledFailed(&'static str),
    TaskRunFailed,
}