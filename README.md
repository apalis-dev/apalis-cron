# apalis-cron

`apalis-cron` is a flexible and extensible Rust library for scheduling and running cron jobs within the `apalis` ecosystem. It enables developers to define jobs using cron expressions, natural language routines, or custom schedules, and provides robust features for persistence, retries, concurrency, and observability.

## Features

- **Cron-based Scheduling**: Use standard cron expressions to define your job schedules.
- **Timezone Support**: Schedule jobs in any timezone.
- **Persistence**: Persist cron jobs to a storage backend (e.g., Postgres, MySQL, SQLite) to ensure they are not lost on restart and can be distributed across multiple workers.
- **Extensibility**: Easily add custom middleware and services.

### Middleware support

`apalis-cron` is built on top of `apalis` and `tower`.
This means you can leverage the full power of workers and middleware, including:

- **Tracing**: For observing the execution of your cron jobs.
- **Retries**: To handle transient failures with configurable backoff strategies.
- **Concurrency**: To control how many instances of a job can run simultaneously.
- **Load-shedding**: To prevent your system from being overloaded by slow cron jobs.

## Examples

### Using `cron` crate

```rust,no_run
use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::{CronStream, Tick};
use cron::Schedule;
use std::str::FromStr;

async fn handle_tick(tick: Tick, data: Data<usize>) {
    // Do something with the current tick
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
```

### Using the builder pattern

```rust,no_run
use apalis::{layers::retry::RetryPolicy, prelude::*};
use apalis_cron::{CronStream, Tick, builder::schedule};
use chrono::Local;

async fn handle_tick(tick: Tick<Local>, data: Data<usize>) -> Result<(), BoxDynError> {
    println!("Handling tick: {:?} with data: {:?}", tick, data);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let schedule = schedule().each().day().at("9:30").build();
    let backend = CronStream::new_with_timezone(schedule, Local);
    let worker = WorkerBuilder::new("morning-cereal")
        .backend(backend)
        .retry(RetryPolicy::retries(5))
        .data(42usize)
        .build(handle_tick);

    worker.run().await?;
    Ok(())
}
```

### Using the `english-to-cron` crate

```rust,no_run
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
```

## Persistence

Sometimes we may want to persist cron jobs for several reasons:

- Distribute cronjobs between multiple servers
- Store the results of the cronjob
- Prevent task skipping in the case of a restart

```rust,no_run
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
```

## Implementing `Schedule`

You can customize the way ticks are provided by implementing your own `Schedule`;

```rust,no_run
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
```


## Observability

You can track your jobs using [apalis-board](https://github.com/apalis-dev/apalis-board).
![Task](https://github.com/apalis-dev/apalis-board/raw/main/screenshots/task.png)

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our development process and how to submit pull requests.

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md) code of conduct.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
