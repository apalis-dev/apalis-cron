use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::{CronStream, Tick};
use apalis_sqlite::{SqlitePool, SqliteStorage};
use cron::Schedule;
use std::str::FromStr;

async fn handle_tick(tick: Tick, data: Data<usize>) {
    // Do something with the current tick
}

#[tokio::main]
async fn main() {
    let schedule = Schedule::from_str("@daily").unwrap();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    SqliteStorage::setup(&pool)
        .await
        .expect("unable to run migrations for sqlite");
    let sqlite = SqliteStorage::new(&pool);

    let cron = CronStream::new(schedule);
    let backend = cron.pipe_to(sqlite);

    let worker = WorkerBuilder::new("morning-cereal")
        .backend(backend)
        .retry(RetryPolicy::retries(5))
        .data(42usize)
        .build(handle_tick);

    worker.run().await.unwrap();
}
