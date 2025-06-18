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
        return format!("{}s ago", diff);
    } else if diff < 3600 {
        return format!("{}m ago", diff / 60);
    } else if diff < 86400 {
        return format!("{}h ago", diff / 3600);
    } else if diff < 604800 {
        return format!("{}d ago", diff / 86400);
    } else {
        return format!("{}w ago", diff / 604800);
    }
}
