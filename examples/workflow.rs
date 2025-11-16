use std::str::FromStr;
use std::time::Duration;

use apalis::{prelude::*};
use apalis_cron::{CronStream, Tick};
use apalis_sqlite::{SqlitePool, SqliteStorage, fetcher::SqliteFetcher};
use apalis_workflow::*;
use cron::Schedule;
use futures_util::{SinkExt, stream::TryStreamExt};

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let schedule = Schedule::from_str("@daily").unwrap();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    SqliteStorage::setup(&pool)
        .await
        .expect("unable to run migrations for sqlite");
    let sqlite: SqliteStorage<Vec<u8>, json::JsonCodec<Vec<u8>>, SqliteFetcher> =
        SqliteStorage::new(&pool);

    let mut cron = CronStream::new(schedule)
        .map_ok(|r| TaskBuilder::new(serde_json::to_vec(&r).unwrap()).build())
        .map_err(|e| panic!("cron stream error: {}", e));

    let mut backend = sqlite.clone();

    tokio::spawn(async move {
        backend.send_all(&mut cron).await.unwrap();
    });

    let workflow = Workflow::new("daily-tasks")
        .and_then(|tick: Tick, _data: Data<usize>| async move {
            println!("Starting workflow with for tick: {:?}", tick);
            // First task
            Ok::<_, BoxDynError>(format!("Hello from tick at {}", tick.get_timestamp()))
        })
        .delay_for(Duration::from_secs(60 * 60))
        .and_then(|res: String| async move {
            // Second task, using result from first
            println!("{}", res);
            Ok::<(), BoxDynError>(())
        });

    let worker = WorkerBuilder::new("morning-cereal")
        .backend(sqlite)
        // .retry(RetryPolicy::retries(5)) // Workflow currently does not support retries yet coz its not clonable
        .data(42usize)
        .build(workflow);

    worker.run().await?;
    Ok(())
}
