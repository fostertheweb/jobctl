use std::process::Command;

pub fn is_job_suspended(pid: u32) -> std::io::Result<bool> {
    let output = Command::new("jobs").arg("-l").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(stdout.contains(&pid.to_string()))
}
