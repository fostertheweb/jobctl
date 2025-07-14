use clap::Parser;
use jobctl::cli::{Cli, Commands, ZSH};
use jobctl::sessions::{ClientRequest, ServerResponse};
use jobctl::utils::{build_fzf_input, run_fzf_cmd};
use std::{env, process};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

use jobctl::ClientError;

fn handle_response(response: Result<serde_json::Value, ClientError>) {
    match response {
        Ok(res) => {
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

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
                if dir.is_some() {
                    match response {
                        Ok(res) => match serde_json::from_value::<ServerResponse>(res) {
                            Ok(ServerResponse::ListJobs { jobs }) => {
                                let (jobs_map, input) = build_fzf_input(jobs);

                                if let Ok(selected) = run_fzf_cmd(&input) {
                                    let job_number = jobs_map
                                        .iter()
                                        .find(|(_, v)| *v == &selected)
                                        .map(|(k, _)| k);
                                    if let Some(job) = job_number {
                                        println!("fg %{}", job)
                                    } else {
                                        eprintln!("no job ID found in line: {}", selected);
                                    }
                                }
                            }
                            Ok(_) => eprintln!("Error: Unexpected response format"),
                            Err(e) => eprintln!("Error parsing response: {}", e),
                        },
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            process::exit(1);
                        }
                    }
                    process::exit(0);
                } else {
                    todo!("Pass sessions to fzf");
                }
            }

            handle_response(response);
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
            handle_response(response);
        }
        Some(Commands::Run { command }) => {
            let request = ClientRequest {
                action: Commands::Run {
                    command: command.to_string(),
                },
                cwd,
            };
            let response = jobctl::sessions::send_request(request, Some(true));
            handle_response(response);
        }
        Some(Commands::Kill) => {
            let request = ClientRequest {
                action: Commands::Kill,
                cwd,
            };
            let response = jobctl::sessions::send_request(request, None);
            handle_response(response);
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
