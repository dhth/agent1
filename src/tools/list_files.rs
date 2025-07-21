use ignore::Walk;

pub fn list_files(path: &str) -> anyhow::Result<Vec<String>> {
    let metadata = std::fs::metadata(path)?;
    if !metadata.is_dir() {
        anyhow::bail!("provided path is not a directory");
    }

    let mut files = Vec::new();
    for result in Walk::new(path) {
        match result {
            Ok(entry) => {
                // TODO: handle this error
                if let Ok(m) = entry.metadata() {
                    if m.is_file() {
                        files.push(entry.path().to_string_lossy().to_string());
                    }
                }
            }
            Err(e) => {
                anyhow::bail!("couldn't list files: {e}");
            }
        }
    }

    Ok(files)
}
