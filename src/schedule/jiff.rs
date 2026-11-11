use jiff_cron::jiff::{Timestamp, tz::TimeZone};

use crate::{Tick, schedule::Schedule};

impl Schedule<TimeZone> for jiff_cron::Schedule {
    fn next_tick(&mut self, timezone: &TimeZone) -> Option<Tick<TimeZone>> {
        self.upcoming(timezone.clone())
            .next()
            .map(|d| d.timestamp().as_second() as _)
            .map(Tick::new)
    }

    fn next_tick_after(
        &mut self,
        tick: &Tick<TimeZone>,
        timezone: &TimeZone,
    ) -> Option<Tick<TimeZone>> {
        // `after` is exclusive: the search starts past `tick`.
        let after = Timestamp::from_second(i64::try_from(tick.get_timestamp()).ok()?)
            .ok()?
            .to_zoned(timezone.clone());
        self.after(after)
            .next()
            .map(|d| d.timestamp().as_second() as _)
            .map(Tick::new)
    }
}

/// Re-export jiff_cron::Schedule as JiffSchedule for convenience
pub use jiff_cron::Schedule as JiffSchedule;

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

    /// 2026-11-10T02:20:00Z.
    const TWENTY_PAST: u64 = 1_794_277_200;

    fn next_after(expression: &str, tick: u64) -> Option<u64> {
        let mut schedule = Schedule::from_str(expression).unwrap();
        crate::schedule::Schedule::next_tick_after(&mut schedule, &Tick::new(tick), &TimeZone::UTC)
            .map(|t| t.get_timestamp())
    }

    #[test]
    fn next_tick_after_skips_the_tick_itself() {
        assert_eq!(
            next_after("0 */10 * * * *", TWENTY_PAST),
            Some(TWENTY_PAST + 600)
        );
    }

    #[test]
    fn next_tick_after_returns_a_tick_one_second_later() {
        assert_eq!(
            next_after("0 */10 * * * *", TWENTY_PAST - 1),
            Some(TWENTY_PAST)
        );
    }

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
