use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::SystemTime,
};

use apalis_core::{
    backend::{Backend, BackendConfig, TryNewBackend, finalize::Ephemeral},
    features_table,
    layers::Identity,
    task::{Task, builder::TaskBuilder, task_id::TaskId},
    timer::Delay,
    worker::context::WorkerContext,
};
use futures_util::Stream;
use ulid::Ulid;

use crate::{config::Config, error::Error, schedule::Schedule, tick::Tick, timezone::Utc};

/// A backend that produces tasks based on a cron schedule.
#[doc = features_table! {
    setup = "unreachable!();",
    TaskSink => not_supported("You cannot push tasks to a cron scheduler"),
}]
#[derive(Debug)]
pub struct CronScheduler<S, Tz = Utc> {
    config: Config<S, Tz>,
    next_tick: Option<Tick<Tz>>,
    delay: Option<Delay>,
}

impl<S> CronScheduler<S, Utc> {
    /// Build a new cron scheduler from a [Schedule] using the [Utc] timezone
    pub const fn new(schedule: S) -> CronScheduler<S> {
        CronScheduler {
            config: Config::new(schedule, Utc),
            next_tick: None,
            delay: None,
        }
    }
}

impl<Schedule: Clone, Timezone: Clone> Clone for CronScheduler<Schedule, Timezone> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            next_tick: None,
            delay: None,
        }
    }
}

impl<S: Schedule<Tz>, Tz> CronScheduler<S, Tz> {
    /// Convert the cron scheduler into a stream of ticks
    pub fn into_stream(self) -> impl Stream<Item = Result<Tick<Tz>, Error>>
    where
        Self: Backend<Task = Task<Tick<Tz>>, Error = Error> + BackendConfig<Args = Tick<Tz>>,
    {
        let mut cron = self;
        futures_util::stream::poll_fn(move |cx| {
            match cron.poll_next(cx, &WorkerContext::new("cron-scheduler")) {
                Poll::Ready(Some(Ok(task))) => Poll::Ready(Some(Ok(task.args))),
                Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        })
    }
}

impl<S, T> CronScheduler<S, T> {
    /// Build a new cron with specific timezone
    pub fn with_timezone<Tz>(self, timezone: Tz) -> CronScheduler<S, Tz> {
        CronScheduler {
            config: Config::new(self.config.schedule, timezone),
            next_tick: None,
            delay: None,
        }
    }
}

impl<S, Tz> Backend for CronScheduler<S, Tz>
where
    S: Schedule<Tz>,
    Tz: Debug + Clone,
{
    type Task = Task<Tick<Tz>>;
    type Error = Error;

    fn poll_ready(
        &mut self,
        _: &mut Context<'_>,
        _: &WorkerContext,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_next(
        &mut self,
        cx: &mut Context<'_>,
        _: &WorkerContext,
    ) -> Poll<Option<Result<Self::Task, Self::Error>>> {
        tracing::trace!(
            has_next_tick = self.next_tick.is_some(),
            has_delay = self.delay.is_some(),
            "polling cron scheduler"
        );

        let tz = self.config.timezone().clone();
        let next_tick = {
            if self.next_tick.is_none() {
                tracing::trace!("calculating next cron tick");

                self.next_tick = self.config.schedule.next_tick(&tz);

                if self.next_tick.is_none() {
                    tracing::debug!("cron schedule exhausted");
                    return Poll::Ready(None);
                }
            }

            if self.delay.is_none() {
                let next = self.next_tick.as_ref().unwrap();
                let now = SystemTime::now();

                tracing::trace!(
                    tick = ?next,
                    now = ?now,
                    "creating delay until next cron tick"
                );

                let duration = match next.signed_duration_since(now) {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::warn!(
                            tick = ?next,
                            error = ?e,
                            "cron tick is out of range"
                        );

                        return Poll::Ready(Some(Err(Error::OutOfRange {
                            duration: e,
                            tick: next.get_timestamp(),
                        })));
                    }
                };

                tracing::trace!(?duration, "cron delay created");

                self.delay = Some(Delay::new(duration));
            }

            match Pin::new(self.delay.as_mut().unwrap()).poll(cx) {
                Poll::Pending => {
                    tracing::trace!("cron delay pending");
                    Poll::Pending
                }

                Poll::Ready(()) => {
                    let fired = self.next_tick.take().unwrap();

                    tracing::debug!(
                        tick = ?fired,
                        "cron tick fired"
                    );
                    self.delay = None;

                    self.next_tick = self.config.schedule.next_tick(&tz);

                    tracing::trace!(
                        next_tick = ?self.next_tick,
                        "scheduled next cron tick"
                    );

                    Poll::Ready(Some(Ok(fired)))
                }
            }
        };

        next_tick.map(|opt| {
            opt.map(|res| {
                res.map(|tick| {
                    let timestamp: SystemTime = tick.system_time();
                    let task_id = Ulid::from_datetime(timestamp);

                    tracing::trace!(
                        timestamp = ?timestamp,
                        %task_id,
                        "building cron task"
                    );

                    TaskBuilder::new(tick)
                        .task_id(TaskId::Ulid(task_id))
                        .build()
                })
            })
        })
    }

    fn poll_close(
        &mut self,
        _: &mut Context<'_>,
        _: &WorkerContext,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl<S, Tz> BackendConfig for CronScheduler<S, Tz> {
    type Args = Tick<Tz>;
    type Config = Config<S, Tz>;
    type Kind = Ephemeral;
    type Layer = Identity;
    type Id = Ulid;

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn middleware(&mut self, _: &mut WorkerContext) -> Self::Layer {
        Identity::new()
    }
}

impl<S: Schedule<Tz>, Tz> TryNewBackend for CronScheduler<S, Tz>
where
    Self: Backend + BackendConfig<Args = Tick<Tz>, Config = Config<S, Tz>>,
{
    type Backend = Self;
    fn try_new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(CronScheduler {
            config,
            delay: None,
            next_tick: None,
        })
    }
}
