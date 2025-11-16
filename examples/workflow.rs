use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::{CronStream, Tick, builder::schedule};

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let schedule = schedule().each().day().build();

    let worker = WorkerBuilder::new("morning-cereal")
        .backend(CronStream::new(schedule))
        .retry(RetryPolicy::retries(5))
        .data(42usize)
        .build(handle_tick);

    worker.run().await?;
    Ok(())
}

async fn handle_tick(tick: Tick, data: Data<usize>) -> Result<(), BoxDynError> {
    println!("Handling tick: {:?} with data: {:?}", tick, data);
    Ok(())
}
