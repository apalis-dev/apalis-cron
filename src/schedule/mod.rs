use crate::Tick;
/// Builder for creating schedules via a fluent API
#[cfg(feature = "builder")]
pub mod builder;

/// Schedule using the `cron` crate
#[cfg(feature = "cron")]
pub mod cron;

/// Schedule using the `jiff_cron` crate
#[cfg(feature = "jiff")]
pub mod jiff;

/// Schedule using `english-to-cron` crate
#[cfg(feature = "english")]
pub mod english;

/// A trait representing a schedule that can compute the next tick.
pub trait Schedule<Tz> {
    /// Returns the next scheduled tick or `None` if there are no more ticks.
    fn next_tick(&mut self, tz: &Tz) -> Option<Tick<Tz>>;

    /// Returns the first tick strictly after `tick`, or `None` if there are no more ticks.
    ///
    /// [`CronScheduler`](crate::CronScheduler) calls this once `tick` has fired, so the
    /// next tick does not depend on whether the timer fired a little before or after the
    /// wall clock reached `tick`. Computing it from the current time instead can hand back
    /// `tick` itself when the timer fires early.
    ///
    /// The default falls back to [`Schedule::next_tick`]. Implement it for schedules that
    /// can compute the next tick from a given instant.
    fn next_tick_after(&mut self, tick: &Tick<Tz>, tz: &Tz) -> Option<Tick<Tz>> {
        let _ = tick;
        self.next_tick(tz)
    }
}

#[cfg(test)]
mod tests {
    use apalis_core::{
        backend::memory::MemoryStorage,
        error::BoxDynError,
        task::{builder::TaskBuilder, task_id::TaskId},
        worker::{builder::WorkerBuilder, event::Event, ext::event_listener::EventListenerExt},
    };
    use cron::Schedule;
    use futures_util::{SinkExt, stream};

    use crate::tick::Tick;

    use std::{str::FromStr, time::Duration};

    #[tokio::test]
    async fn eager_worker() {
        let schedule = Schedule::from_str("1/1 * * * * *").unwrap();
        let five_ticks = schedule.upcoming(chrono::Utc).take(5);
        let mut tasks = stream::iter(five_ticks.map(|s| {
            let ts = s.timestamp() as u64;
            let task = TaskBuilder::new(Tick::new(ts)).run_at_timestamp(ts).build();
            Ok(task)
        }));

        let mut memory = MemoryStorage::new();

        memory.send_all(&mut tasks).await.unwrap();

        async fn send_reminder(job: Tick, id: TaskId) -> Result<(), BoxDynError> {
            println!("Running cronjob for timestamp: {:?} with id {}", job, id);
            tokio::time::sleep(Duration::from_secs(1)).await;
            Err("All failing".into())
        }

        let worker = WorkerBuilder::new("rango-tango")
            .backend(memory)
            .on_event(move |ctx, ev| {
                println!("{:?}", ev);
                let ctx = ctx.clone();
                if matches!(ev, Event::Error(_)) {
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        ctx.stop().unwrap();
                    });
                }
            })
            .build(send_reminder);
        worker.run().await.unwrap();
    }
}
