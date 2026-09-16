use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::{CronScheduler, Tick, builder::schedule};

async fn handle_tick(tick: Tick, data: Data<usize>) -> Result<(), BoxDynError> {
    println!("Handling tick: {:?} with data: {:?}", tick, data);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let schedule = schedule().every(1).minutes().build();
    let backend = CronScheduler::new(schedule);
    let worker = WorkerBuilder::new("morning-cereal")
        .backend(backend)
        .retry(RetryPolicy::retries(5))
        .data(42usize)
        .build(handle_tick);

    worker.run().await?;
    Ok(())
}
