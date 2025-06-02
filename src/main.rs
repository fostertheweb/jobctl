use clap::Parser;
use jobctl::cli::{Cli, Commands, ZSH};
use jobctl::sessions::ClientRequest;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir().expect("Failed to get current directory");

    match &cli.command {
        Some(Commands::List) => {
            let request = ClientRequest {
                action: Commands::List,
                cwd,
            };
            let response = jobctl::sessions::send_request(request, None);
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Some(Commands::Register { pid, .. }) => {
            let sys = System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
            );
            let process = sys
                .process(Pid::from(*pid as usize))
                .expect(format!("Did not find process with pid {}", pid).as_str());
            let request = ClientRequest {
                action: Commands::Register {
                    pid: *pid,
                    command: process.name().to_string_lossy().into(),
                    suspended: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
                cwd,
            };
            let response = jobctl::sessions::send_request(request, Some(true));
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Some(Commands::Init { shell }) => {
            let output = match shell.as_str() {
                "zsh" => ZSH,
                _ => "Shell not supported.",
            };
            println!("{}", output);
        }
        None => {}
    }

    Ok(())
}
