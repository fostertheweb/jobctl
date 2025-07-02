use clap::Parser;
use jobctl::utils::time_ago;

use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
};
use tracing::{error, info};

use jobctl::cli::Commands;
use jobctl::sessions::{ClientRequest, Job, JobOutput, ServerResponse, Session, cleanup_sessions};

fn handle_client(mut stream: UnixStream, store: &Arc<Mutex<Vec<Session>>>) -> std::io::Result<()> {
    let mut kill_after_response = false;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    reader.read_line(&mut line)?;

    let req: ClientRequest = match serde_json::from_str(&line) {
        Ok(r) => {
            info!("{}", serde_json::to_string_pretty(&r).unwrap());
            r
        }
        Err(e) => {
            let err = ServerResponse::Error {
                message: format!("Request Error: {}", e),
            };
            let payload = serde_json::to_string(&err).unwrap();
            writeln!(stream, "{}", payload)?;
            return Ok(());
        }
    };

    info!("Processing action: {:?}", req.action);

    let response = match req.action {
        Commands::List { dir, fzf: _ } => {
            let sessions = cleanup_sessions(store);

            match dir {
                Some(directory) => {
                    let directory = PathBuf::from(directory);
                    let session = sessions
                        .iter()
                        .find(|s| s.directory == directory)
                        .expect("No jobs found for directory");
                    let jobs = session
                        .jobs
                        .clone()
                        .iter()
                        .map(|job| JobOutput {
                            pid: job.pid,
                            command: job.command.clone(),
                            number: job.number,
                            suspended: time_ago(job.suspended),
                        })
                        .collect();
                    ServerResponse::ListJobs { jobs }
                }
                _ => ServerResponse::ListSessions { sessions },
            }
        }
        Commands::Register {
            pid,
            number,
            command,
        } => {
            let job = Job {
                pid,
                number,
                command,
                suspended: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::new(0, 0))
                    .as_secs(),
            };

            info!("Creating new job: {:?}", job);

            let mut sessions = store.lock().unwrap();
            if let Some(session) = sessions.iter_mut().find(|s| s.directory == req.cwd) {
                if session.jobs.iter().all(|j| j.pid != job.pid) {
                    session.jobs.push(job.clone());
                    info!("Adding to job to session: {:?}", session);
                } else {
                    info!(
                        "Job with PID {} already exists in session: {:?}",
                        job.pid, session
                    );
                }
            } else {
                let session = Session {
                    jobs: vec![job.clone()],
                    directory: req.cwd,
                };

                info!("No session found, creating session: {:?}", session);

                sessions.push(session);
            }

            ServerResponse::Register { job }
        }
        Commands::Kill => {
            kill_after_response = true;
            ServerResponse::Kill
        }
        Commands::Run { command } => {
            // Spawn the command as a background process
            let child = match ProcessCommand::new("sh").arg("-c").arg(&command).spawn() {
                Ok(child) => child,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to spawn process: {}", e),
                    ));
                }
            };

            let pid = child.id();

            let job = Job {
                pid,
                number: 0,
                command: command.clone(),
                suspended: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::new(0, 0))
                    .as_secs(),
            };

            info!("Spawning new job: {:?}", job);

            let mut sessions = store.lock().unwrap();
            if let Some(session) = sessions.iter_mut().find(|s| s.directory == req.cwd) {
                if session.jobs.iter().all(|j| j.pid != job.pid) {
                    session.jobs.push(job.clone());
                    info!("Adding spawned job to session: {:?}", session);
                } else {
                    info!(
                        "Job with PID {} already exists in session: {:?}",
                        job.pid, session
                    );
                }
            } else {
                let session = Session {
                    jobs: vec![job.clone()],
                    directory: req.cwd,
                };

                info!(
                    "No session found, creating session for spawned job: {:?}",
                    session
                );

                sessions.push(session);
            }

            ServerResponse::Register { job }
        }
        _ => todo!(),
    };

    let payload = serde_json::to_string(&response).unwrap();
    writeln!(stream, "{}", payload)?;

    if kill_after_response {
        exit(0);
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = jobctl::cli::ServerArgs::parse();

    let level = match args.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt().with_max_level(level).init();

    let uid = unsafe { libc::getuid() };
    let socket_path = format!("/tmp/jobctl-{}.sock", uid);
    if let Err(e) = fs::remove_file(PathBuf::from(&socket_path)) {
        match e.kind() {
            std::io::ErrorKind::NotFound => {}
            _ => {
                error!("IO Error: Failed to remove `{}`", &socket_path);
                error!("IO Error: {}", e);
            }
        }
    }
    let listener = match UnixListener::bind(&socket_path) {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind socket (another instance running?): {}", e);
            std::process::exit(1);
        }
    };
    let store: Arc<Mutex<Vec<Session>>> = Arc::new(Mutex::new(vec![]));

    info!("Server Started:  {}", socket_path);

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let store = Arc::clone(&store);
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, &store) {
                        error!("Client Error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Socket Listener Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
