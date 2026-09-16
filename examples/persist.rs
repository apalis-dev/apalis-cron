use apalis::{
    layers::{retry::RetryPolicy, tracing},
    prelude::*,
};

use ::tracing::Instrument;
use apalis_codec::msgpack::MsgPackCodec;
use apalis_cron::{CronScheduler, Tick};
use apalis_sqlite::{SqlitePool, SqliteStorage};
use cron::Schedule;
use sqlx::{Row, query};
use std::str::FromStr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn handle_tick(tick: Tick, pool: Data<SqlitePool>) -> Result<(), BoxDynError> {
    tracing::info!("Tick received at {}", tick.get_timestamp());
    let jobs_count = query("SELECT count(*) FROM Jobs").fetch_one(&*pool).await?;
    tracing::info!("Number of jobs: {}", jobs_count.get::<i64, _>(0));
    Ok(())
}

#[tokio::main]
async fn main() {
    use tracing_subscriber::{EnvFilter, fmt};
    let fmt_layer = fmt::layer();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug,sqlx=off"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
    // Every second schedule
    let schedule = Schedule::from_str("1/1 * * * * *").unwrap();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    SqliteStorage::setup(&pool)
        .await
        .expect("unable to run migrations for sqlite");

    let worker = "morning-cereal";

    let worker_span = tracing::info_span!(
        "worker",
        worker = %worker,
    );

    let pipe_span = tracing::info_span!(
        parent: &worker_span,
        "pipe",
    );

    let sqlite_span = tracing::info_span!(
        parent: &pipe_span,
        "sqlite",
    );

    let cron_span = tracing::info_span!(
        parent: &pipe_span,
        "cron",
    );

    let sqlite = SqliteStorage::new(&pool)
        .with_codec(MsgPackCodec::default())
        .instrumented(sqlite_span);

    let cron = CronScheduler::new(schedule).instrumented(cron_span);

    let backend = cron.pipe_to(sqlite).instrumented(pipe_span);

    WorkerBuilder::new(worker)
        .backend(backend)
        .data(pool)
        .retry(RetryPolicy::retries(5))
        .enable_tracing()
        .on_event(|_, e| tracing::info!("{e:?}"))
        .build(handle_tick)
        .run_until(tokio::signal::ctrl_c())
        .instrument(worker_span)
        .await
        .unwrap();
}
