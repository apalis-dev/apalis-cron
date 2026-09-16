use apalis::{
    config::{WorkerConfig, WorkerFromConfig},
    prelude::*,
};
use apalis_cron::{CronScheduler, Tick, english::EnglishRoutine};
use chrono_tz::Tz;

async fn handle_tick(tick: Tick<Tz>, _: Data<usize>) {
    println!("Tick received at {}", tick.get_timestamp());
}

const CONFIG: &str = r#"
{
  "name": "simple-worker",
  "backend": {
    "schedule": "every 1 second",
    "timezone": "Africa/Johannesburg"
  },
  "middleware": [
    {
      "Timeout": {
        "duration": {
          "secs": 5,
          "nanos": 0
        }
      }
    },
    "CatchPanic",
    "Tracing",
    {
      "Retries": {
        "max": 10
      }
    }
  ]
}
"#;

type Config = WorkerConfig<CronScheduler<EnglishRoutine, Tz>>;

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let config: Config = serde_json::from_str(CONFIG).unwrap();

    let worker = WorkerBuilder::try_config(config)?
        .data(4usize)
        .on_event(|_, e| tracing::info!("{e:?}"))
        .build(handle_tick);

    worker.run().await?;

    Ok(())
}
