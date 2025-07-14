use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sysinfo::{Pid, ProcessRefreshKind, ProcessStatus, RefreshKind, System};

use crate::sessions::{JobOutput, Session};

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

pub fn build_fzf_sessions_input(sessions: Vec<Session>) -> String {
    let mut input = String::new();

    sessions
        .iter()
        .for_each(|session| input.push_str(&format!("{}\n", session.directory.display())));

    input
}

pub fn build_fzf_jobs_input(jobs: Vec<JobOutput>) -> (HashMap<u8, String>, String) {
    let mut jobs_map = HashMap::new();

    jobs.iter().for_each(|job| {
        jobs_map.insert(
            job.number,
            format!(
                "[{}:{}] - {}, {} \n",
                job.number, job.pid, job.command, job.suspended
            ),
        );
    });

    let input = jobs_map
        .clone()
        .into_values()
        .collect::<Vec<String>>()
        .join("\n");

    (jobs_map, input)
}

pub fn run_fzf_cmd(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("fzf selection cancelled or failed".into())
    }
}
