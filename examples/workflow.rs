use std::str::FromStr;
use std::time::Duration;

use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_codec::msgpack::MsgPackCodec;
use apalis_core::task::context::TaskContext;
use apalis_cron::{CronScheduler, Tick};
use apalis_sqlite::{SqlitePool, SqliteStorage};
use apalis_workflow::*;
use cron::Schedule;

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let schedule = Schedule::from_str("1/1 * * * * *").unwrap();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    SqliteStorage::setup(&pool)
        .await
        .expect("unable to run migrations for sqlite");
    let sqlite = SqliteStorage::new(&pool).with_codec(MsgPackCodec::default());

    let cron = CronScheduler::new(schedule);

    let backend = cron.pipe_to(sqlite);

    let workflow = SteppedFlow::new("daily-tasks")
        .and_then(
            |tick: Tick, _pool: Data<SqlitePool>, _ctx: TaskContext| async move {
                // _pool.begin()
                println!("Starting workflow with for tick: {:?}", tick);
                // First task
                Ok::<_, BoxDynError>(format!("Hello from tick at {}", tick.get_timestamp()))
            },
        )
        .delay_for(Duration::from_secs(60 * 60))
        .and_then(|res: String| async move {
            // Second task, using result from first
            println!("{}", res);
            Ok::<(), BoxDynError>(())
        });

    let worker = WorkerBuilder::new("morning-cereal")
        .backend(backend)
        .retry(RetryPolicy::retries(5))
        .data(pool)
        .build(workflow);

    worker.run().await?;
    Ok(())
}
