mod country;
mod error;
mod model;
mod monitor;
mod task;

use country::Country;
pub use error::Error;
use futures::{stream::futures_unordered::FuturesUnordered, StreamExt};
use log::error;
use model::{Config, Variant};
use monitor::Monitor;
use std::{collections::HashMap, fs::File};
use tokio::sync::broadcast::{self, Sender};

use crate::task::Task;

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init_timed();

    let file = File::open("config.json")?;
    let config: Config = serde_json::from_reader(file)?;

    let mut tasks = FuturesUnordered::new();

    for task_config in config.tasks {
        let mut monitors: HashMap<Country, Sender<(i64, Vec<Variant>)>> = HashMap::new();

        let countries = task_config
            .profiles
            .iter()
            .map(|profile| profile.delivery.country.clone())
            .collect::<Vec<_>>();

        for country in countries {
            let (sender, _) = broadcast::channel::<(i64, Vec<Variant>)>(32);
            let mut monitor = Monitor::new(task_config.product.clone(), &country, sender.clone())?;
            monitors.insert(country, sender);

            let handle = tokio::task::spawn(async move { monitor.start().await });

            tasks.push(handle);
        }

        for profile in task_config.profiles {
            let sender = monitors.get(&profile.delivery.country).unwrap();
            let mut task = Task::new(profile, sender.subscribe())?;
            let handle = tokio::task::spawn(async move { task.start().await });

            tasks.push(handle);
        }
    }

    while let Some(join) = tasks.next().await {
        if let Err(why) = join.unwrap() {
            error!("message={},  error=\"task failure\"", why)
        }
    }

    Ok(())
}
