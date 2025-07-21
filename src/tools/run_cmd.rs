use anyhow::Context;
use std::process::Command;

const BLOCKED_CMD_PATTERNS: [&str; 23] = [
    "rm -rf",
    "sudo",
    "curl",
    "wget",
    "dd if=",
    "mkfs",
    "fdisk",
    "format",
    "deltree",
    "rmdir /s",
    "nc ",
    "netcat",
    "telnet",
    "ssh-keygen",
    "passwd",
    "useradd",
    "userdel",
    "chmod 777",
    "chown root",
    "python -c",
    "perl -e",
    "ruby -e",
    "node -e",
];

pub fn run_cmd(cmd: &str) -> anyhow::Result<String> {
    if cmd.is_empty() {
        anyhow::bail!("command is empty")
    }

    for pattern in BLOCKED_CMD_PATTERNS {
        if cmd.contains(pattern) {
            anyhow::bail!(
                "command contains a forbidden pattern '{pattern}'; blocked for security reasons"
            );
        }
    }

    // TODO: make it cross-platform, use "sh" if bash unavailable
    let output = Command::new("bash")
        .args(["-c", cmd])
        .output()
        .context("couldn't run command")?;

    let mut combined_output = vec![];
    combined_output.push("stdout:".to_string());
    combined_output.push(String::from_utf8(output.stdout).context("couldn't get command stdout")?);
    combined_output.push("\n---\n".to_string());
    combined_output.push("stderr:\n".to_string());
    combined_output.push(String::from_utf8(output.stderr).context("couldn't get command stderr")?);

    let combined_output = combined_output.join("\n");

    if !output.status.success() {
        anyhow::bail!("command failed:\n{}", combined_output);
    }

    Ok(combined_output)
}
