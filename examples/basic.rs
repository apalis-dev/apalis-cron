use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::{CronStream, Tick};
use cron::Schedule;
use std::str::FromStr;

async fn handle_tick(tick: Tick, data: Data<usize>) {
    assert!(*data == 42usize);
    println!("Tick received at {}", tick.get_timestamp());
}

#[tokio::main]
async fn main() {
    let schedule = Schedule::from_str("@daily").unwrap();

    let worker = WorkerBuilder::new("morning-cereal")
        .backend(CronStream::new(schedule))
        .retry(RetryPolicy::retries(5))
        .data(42usize)
        .build(handle_tick);

    worker.run().await.unwrap();
}
