use std::io::{Write};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sysinfo::{Pid, ProcessRefreshKind, ProcessStatus, RefreshKind, System};

pub fn is_job_suspended(pid: u32) -> bool {
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    match sys.process(Pid::from(pid as usize)) {
        Some(process) => match process.status() {
            ProcessStatus::Stop => true,
            ProcessStatus::Unknown(exit_code) => exit_code == 0,
            _ => false,
        },
        None => false,
    }
}

pub fn time_ago(timestamp: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::new(0, 0))
        .as_secs();

    let diff = now - timestamp;

    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        format!("{}w ago", diff / 604800)
    }
}

pub fn run_fzf_with_input(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write input to stdin
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin.write_all(input.as_bytes())?;
        // stdin is dropped here, closing the pipe
    }

    // Wait for fzf to finish and capture output
    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("fzf selection cancelled or failed".into())
    }
}

pub fn between_bracket_and_colon(s: &str) -> Option<&str> {
    // find the first '['
    let start = s.find('[')? + 1;
    // find the first ':' *after* that '['
    let end = s[start..].find(':')? + start;
    // slice out everything between them
    Some(&s[start..end])
}
