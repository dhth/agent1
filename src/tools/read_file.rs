use anyhow::Context;

pub fn read_file(path: &str) -> anyhow::Result<String> {
    std::fs::read_to_string(path).context("couldn't read file contents")
}
