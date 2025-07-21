mod agent;
mod log;
mod tools;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    log::setup_logging().context("couldn't set up logging")?;
    let client = reqwest::blocking::Client::new();
    agent::run(client)?;

    Ok(())
}
