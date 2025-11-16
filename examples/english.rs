// Run this example with `cargo run --example english --features=english`

use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::english::EnglishRoutine;
use apalis_cron::{CronStream, Tick};
use std::str::FromStr;

async fn handle_tick(tick: Tick, data: Data<usize>) -> Result<(), BoxDynError> {
    println!("Handling tick: {:?} with data: {:?}", tick, data);
    Ok(())
}

#[tokio::main]
async fn main() {
    let schedule = EnglishRoutine::from_str("every day").unwrap();

    let worker = WorkerBuilder::new("morning-cereal")
        .backend(CronStream::new(schedule))
        .retry(RetryPolicy::retries(5))
        .data(42usize)
        .build(handle_tick);

    worker.run().await.unwrap();
}
