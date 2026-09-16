use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::{CronScheduler, Tick, builder::schedule};
use chrono::Local;

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let schedule = schedule().each().day().at("9:30").build();
    let backend = CronScheduler::new(schedule).with_timezone(Local);
    let worker = WorkerBuilder::new("morning-cereal")
        .backend(backend)
        .retry(RetryPolicy::retries(5))
        .data(42usize)
        .build(handle_tick);

    worker.run().await?;
    Ok(())
}

async fn handle_tick(tick: Tick<Local>, data: Data<usize>) -> Result<(), BoxDynError> {
    println!("Handling tick: {:?} with data: {:?}", tick, data);
    Ok(())
}
