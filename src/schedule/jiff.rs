use jiff_cron::jiff::tz::TimeZone;

use crate::{Tick, schedule::Schedule};

impl Schedule<TimeZone> for jiff_cron::Schedule {
    fn next_tick(&mut self, timezone: &TimeZone) -> Option<Tick<TimeZone>> {
        self.upcoming(timezone.clone())
            .next()
            .map(|d| d.timestamp().as_second() as _)
            .map(Tick::new)
    }
}

#[cfg(test)]
mod tests {
    use apalis_core::{
        backend::ext::BackendExt,
        error::BoxDynError,
        task::task_id::TaskId,
        worker::{builder::WorkerBuilder, event::Event, ext::event_listener::EventListenerExt},
    };
    use jiff_cron::{
        Schedule,
        jiff::tz::{Offset, TimeZone},
    };
    use tracing::info;

    use crate::{backend::CronScheduler, tick::Tick};

    use std::{str::FromStr, time::Duration};

    #[tokio::test]
    async fn basic_worker() {
        let schedule = Schedule::from_str("1/1 * * * * *").unwrap();
        let span = tracing::info_span!("cron");
        let backend = CronScheduler::new(schedule)
            .with_timezone(TimeZone::fixed(Offset::constant(3)))
            .instrumented(span);

        async fn send_reminder(job: Tick<TimeZone>, id: TaskId) -> Result<(), BoxDynError> {
            info!("Running cronjob for timestamp: {:?} with id {}", job, id);
            tokio::time::sleep(Duration::from_secs(1)).await;
            Err("All failing".into())
        }

        let worker = WorkerBuilder::new("rango-tango")
            .backend(backend)
            .on_event(move |ctx, ev| {
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
