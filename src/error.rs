use std::time::Duration;

/// Represents an error emitted by `CronScheduler` polling
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The cron scheduler might not always be polled consistently, such as when the worker is blocked.
    /// If polling is delayed, some ticks may be skipped. When this occurs, an out-of-range error is triggered
    /// because the missed tick is now in the past.
    #[error("Tick out of range for {tick:?} (past duration:  {duration:?})")]
    OutOfRange {
        /// The past duration
        duration: Duration,
        /// The missed tick
        tick: u64,
    },
}
