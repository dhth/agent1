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
    "nc",
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

    // TODO: this is quite naive, improve this
    for pattern in BLOCKED_CMD_PATTERNS {
        if cmd.contains(pattern) {
            anyhow::bail!(
                "command contains a forbidden pattern '{pattern}'; blocked for security reasons"
            );
        }
    }

    // TODO: make it cross-platform, have fallback if bash unavailable
    let output = Command::new("bash")
        .args(["-c", cmd])
        .output()
        .context("couldn't run command")?;

    let combined_output = format!(
        r#"success: {}
exit_code: {}
----- stdout -----
{}
----- stderr -----
{}"#,
        output.status.success(),
        output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or("unknown".to_string()),
        String::from_utf8(output.stdout).context("couldn't get command stdout")?,
        String::from_utf8(output.stderr).context("couldn't get command stderr")?
    );

    Ok(combined_output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn output_of_a_successful_command_is_returned() {
        // GIVEN
        let cmd = "cat src/tools/testdata/sample.txt";

        // WHEN
        let result = run_cmd(cmd).expect("result should've been a success");

        // THEN
        insta::assert_snapshot!(result, @r"
        success: true
        exit_code: 0
        ----- stdout -----
        This file contains 3 lines.
        This is line #2.
        This is line #3.

        ----- stderr -----
        ")
    }

    #[test]
    fn output_of_a_failing_command_is_returned() {
        // GIVEN
        let cmd = "ls nonexistent/directory";

        // WHEN
        let result = run_cmd(cmd).expect("result should've been a success");

        // THEN
        insta::assert_snapshot!(result, @r"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        ls: nonexistent/directory: No such file or directory
        ")
    }

    #[test]
    fn command_with_pipes_can_be_run() {
        // GIVEN
        let cmd = "cat src/tools/testdata/sample.txt | grep '#' | wc -l | xargs";

        // WHEN
        let result = run_cmd(cmd).expect("result should've been a success");

        // THEN
        insta::assert_snapshot!(result, @r"
        success: true
        exit_code: 0
        ----- stdout -----
        2

        ----- stderr -----
        ")
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn running_empty_command_fails() {
        // GIVEN
        // WHEN
        let result = run_cmd("").expect_err("result wasn't an error");

        // THEN
        assert_snapshot!(result, @"command is empty");
    }

    #[test]
    fn command_with_blocked_pattern_is_rejected() {
        // GIVEN
        let cmd = "curl http://127.0.0.1:9999";

        // WHEN
        let result = run_cmd(cmd).expect_err("result wasn't an error");

        // THEN
        assert_snapshot!(result, @"command contains a forbidden pattern 'curl'; blocked for security reasons");
    }
}
