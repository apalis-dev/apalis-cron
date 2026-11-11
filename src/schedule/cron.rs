use crate::{Tick, schedule::Schedule};

impl<Tz: chrono::TimeZone> Schedule<Tz> for cron::Schedule {
    fn next_tick(&mut self, timezone: &Tz) -> Option<Tick<Tz>> {
        self.upcoming(timezone.clone())
            .next()
            .map(|d| d.timestamp() as _)
            .map(Tick::new)
    }

    fn next_tick_after(&mut self, tick: &Tick<Tz>, timezone: &Tz) -> Option<Tick<Tz>> {
        // `after` is exclusive: the search starts one second past `tick`.
        let after = timezone
            .timestamp_opt(i64::try_from(tick.get_timestamp()).ok()?, 0)
            .single()?;
        self.after(&after)
            .next()
            .map(|d| d.timestamp() as _)
            .map(Tick::new)
    }
}

#[cfg(test)]
mod tests {
    use apalis_core::{
        error::BoxDynError,
        task::task_id::TaskId,
        worker::{builder::WorkerBuilder, event::Event, ext::event_listener::EventListenerExt},
    };
    use cron::Schedule;

    use crate::{backend::CronScheduler, tick::Tick};

    use std::{str::FromStr, time::Duration};

    /// 2026-11-10T02:20:00Z.
    const TWENTY_PAST: u64 = 1_794_277_200;

    fn next_after(expression: &str, tick: u64) -> Option<u64> {
        let mut schedule = Schedule::from_str(expression).unwrap();
        crate::schedule::Schedule::next_tick_after(&mut schedule, &Tick::new(tick), &chrono::Utc)
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

    #[tokio::test]
    async fn basic_worker() {
        let schedule = Schedule::from_str("1/1 * * * * *").unwrap();
        let backend = CronScheduler::new(schedule);

        async fn send_reminder(job: Tick, id: TaskId) -> Result<(), BoxDynError> {
            println!("Running cronjob for timestamp: {:?} with id {}", job, id);
            tokio::time::sleep(Duration::from_secs(1)).await;
            Err("All failing".into())
        }

        let worker = WorkerBuilder::new("rango-tango")
            .backend(backend)
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
