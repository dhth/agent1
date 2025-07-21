mod agent;
mod tools;

fn main() -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::new();
    agent::run(client)?;

    Ok(())
}
