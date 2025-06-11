use sysinfo::{Pid, ProcessRefreshKind, ProcessStatus, RefreshKind, System};

pub fn is_job_suspended(pid: u32) -> bool {
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    match sys.process(Pid::from(pid as usize)) {
        Some(process) => match process.status() {
            ProcessStatus::Stop => true,
            _ => false,
        },
        None => false,
    }
}
