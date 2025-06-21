use clap::Parser;
use jobctl::cli::{Cli, Commands, ZSH};
use jobctl::sessions::ClientRequest;
use std::env;
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};
use tracing::info;

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir().expect("Failed to get current directory");

    let level = match cli.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt().with_max_level(level).init();

    match &cli.command {
        Some(Commands::List { dir }) => {
            let request = ClientRequest {
                action: Commands::List { dir: dir.clone() },
                cwd,
            };
            let response = jobctl::sessions::send_request(request, None);
            // This needs to print for parsing by jq
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Some(Commands::Register { pid, number, .. }) => {
            let sys = System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
            );
            let process = sys
                .process(Pid::from(*pid as usize))
                .expect(format!("Did not find process with pid {}", pid).as_str());
            let request = ClientRequest {
                action: Commands::Register {
                    pid: *pid,
                    number: *number,
                    command: process.name().to_string_lossy().into(),
                },
                cwd,
            };
            let response = jobctl::sessions::send_request(request, Some(true));
            info!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Some(Commands::Run { command }) => {
            let request = ClientRequest {
                action: Commands::Run {
                    command: command.to_string(),
                },
                cwd,
            };
            let response = jobctl::sessions::send_request(request, Some(true));
            info!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Some(Commands::Kill) => {
            let request = ClientRequest {
                action: Commands::Kill,
                cwd,
            };
            let response = jobctl::sessions::send_request(request, Some(true));
            info!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Some(Commands::Init { shell }) => {
            // TODO: add bash, fish support
            let output = match shell.as_str() {
                "zsh" => ZSH,
                _ => "Shell not supported.",
            };
            // This needs to print for the shell to evaluate
            println!("{}", output);
        }
        None => {}
    }

    Ok(())
}
