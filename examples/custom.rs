use apalis::prelude::*;
use apalis_cron::{CronStream, Schedule, Tick};
use chrono::{DateTime, Duration, Local, NaiveTime};

/// Daily routine at 8am
#[derive(Debug, Clone)]
struct MyDailyRoutine;

impl Schedule<Local> for MyDailyRoutine {
    fn next_tick(&mut self, _: &Local) -> Option<DateTime<Local>> {
        let now = Local::now();
        // Add 1 day to get tomorrow
        let tomorrow = now.date_naive() + Duration::days(1);

        // Define 8:00 AM as a NaiveTime
        let eight_am = NaiveTime::from_hms_opt(8, 0, 0).unwrap();

        // Combine tomorrow's date with 8:00 AM in local time zone
        let tomorrow_eight_am = tomorrow
            .and_time(eight_am)
            .and_local_timezone(Local)
            .unwrap();

        Some(tomorrow_eight_am)
    }
}

async fn handle_tick(tick: Tick<Local>, data: Data<usize>) -> Result<(), BoxDynError> {
    println!("Handling tick: {:?} with data: {:?}", tick, data);
    Ok(())
}

#[tokio::main]
async fn main() {
    let cron_stream = CronStream::new_with_timezone(MyDailyRoutine, Local);
    let worker = WorkerBuilder::new("morning-cereal")
        .backend(cron_stream)
        .build(handle_tick);

    worker.run().await.unwrap();
}
