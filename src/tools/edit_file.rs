use anyhow::Context;
use std::path::Path;

pub fn edit_file(path: &str, old_str: &str, new_str: &str) -> anyhow::Result<String> {
    if old_str == new_str {
        anyhow::bail!("old_str is equal to new_str")
    }

    let path = Path::new(path);
    match path.metadata() {
        Ok(m) => {
            if !m.is_file() {
                anyhow::bail!("provided path is not a file");
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if !old_str.is_empty() {
                anyhow::bail!("path doesn't exist and old_str is not empty");
            }

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).context("couldn't create directory")?;
            }
            std::fs::write(path, new_str).context("couldn't write to file")?;

            return Ok("file created".to_string());
        }

        Err(e) => return Err(anyhow::anyhow!("couldn't check for file metadata: {e}")),
    }

    if old_str.is_empty() {
        std::fs::write(path, new_str).context("couldn't write to file")?;
        return Ok("entire file modified".to_string());
    }

    let old_contents = std::fs::read_to_string(path).context("couldn't read file")?;
    let new_contents = old_contents.replace(old_str, new_str);

    if old_contents == new_contents {
        anyhow::bail!("nothing changed in file")
    }

    std::fs::write(path, new_contents).context("couldn't write to file")?;

    Ok("file modified".to_string())
}
