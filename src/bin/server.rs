use clap::Parser;
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
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
use jobctl::sessions::{ClientRequest, Job, ServerResponse, cleanup_sessions, encode_path};

fn handle_client(
    mut stream: UnixStream,
    store: &Arc<Mutex<HashMap<String, Vec<Job>>>>,
) -> std::io::Result<()> {
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

    let mut jobs = store.lock().unwrap();
    let response = match req.action {
        Commands::List => ServerResponse::List { jobs: jobs.clone() },
        Commands::Register {
            pid,
            number,
            command,
        } => {
            let key = encode_path(&req.cwd);
            let job = Job {
                cwd: req.cwd,
                pid,
                number,
                command,
                suspended: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::new(0, 0))
                    .as_secs(),
            };

            let session = jobs.entry(key.to_string()).or_insert(vec![]);
            if session.iter().all(|j| j.pid != job.pid) {
                session.push(job.clone());
            }

            ServerResponse::Register { job }
        }
        Commands::Init { shell } => ServerResponse::Init { shell },
    };

    let payload = serde_json::to_string(&response).unwrap();
    writeln!(stream, "{}", payload)?;

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
    let store: Arc<Mutex<HashMap<String, Vec<Job>>>> = Arc::new(Mutex::new(HashMap::new()));

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
