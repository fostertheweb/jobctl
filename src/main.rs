use clap::Parser;
use jobctl::cli::{Cli, Commands, ZSH};
use jobctl::sessions::{ClientRequest, ServerResponse};
use jobctl::utils::{between_bracket_and_colon, run_fzf_with_input};
use std::{env, process};
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
        Some(Commands::List { fzf, dir }) => {
            let request = ClientRequest {
                action: Commands::List {
                    fzf: *fzf,
                    dir: dir.clone(),
                },
                cwd,
            };
            let response = jobctl::sessions::send_request(request, None);

            if *fzf {
                if let Some(_) = dir {
                    let mut input = String::new();
                    let list: ServerResponse = serde_json::from_value(response).unwrap();
                    if let ServerResponse::ListJobs { jobs } = list {
                        jobs.iter().for_each(|job| {
                            input.push_str(&format!(
                                "[{}:{}] - {}, {} \n",
                                job.number, job.pid, job.command, job.suspended
                            ))
                        });
                    }

                    if let Ok(selected) = run_fzf_with_input(&input) {
                        if let Some(job) = between_bracket_and_colon(&selected) {
                            println!("fg %{}", job)
                        } else {
                            eprintln!("no job ID found in line: {}", selected);
                        }
                    }

                    process::exit(0);
                } else {
                    todo!("Pass sessions to fzf");
                }
            }

            // This needs to print for parsing by jq
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Some(Commands::Register { pid, number, .. }) => {
            let sys = System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
            );
            let process = sys
                .process(Pid::from(*pid as usize))
                .unwrap_or_else(|| panic!("Did not find process with pid {}", pid));
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
            let response = jobctl::sessions::send_request(request, None);
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
