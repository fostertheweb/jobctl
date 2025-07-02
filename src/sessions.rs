use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, thread};
use tracing::info;

use crate::cli::Commands;
use crate::utils::is_job_suspended;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Job {
    pub pid: u32,
    pub command: String,
    pub number: u8,
    pub suspended: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct JobOutput {
    pub pid: u32,
    pub command: String,
    pub number: u8,
    pub suspended: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Session {
    pub jobs: Vec<Job>,
    pub directory: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ClientRequest {
    #[serde(flatten)]
    pub action: Commands,
    pub cwd: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ServerResponse {
    ListJobs { jobs: Vec<JobOutput> },
    ListSessions { sessions: Vec<Session> },
    Register { job: Job },
    Kill,
    Error { message: String },
}

pub fn encode_path(path: &Path) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(path.to_string_lossy().as_bytes())
}

pub fn start_server() {
    let exe = env::current_exe().expect("Failed to get executable path");
    let server_path = exe.with_file_name("server");

    let mut child = Command::new(server_path)
        .spawn()
        .expect("Failed to start server.");
    child.wait().expect("Failed to wait on server process");
    thread::sleep(Duration::from_millis(500));
}

pub fn send_request(request: ClientRequest, should_start: Option<bool>) -> Value {
    let should_start = should_start.unwrap_or(false);
    let uid = unsafe { libc::getuid() };
    let socket_path = format!("/tmp/jobctl-{}.sock", uid);
    let mut resp_line = String::new();
    let mut stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(_) => {
            if should_start {
                start_server();
                UnixStream::connect(&socket_path).expect("Failed to connect to server")
            } else {
                println!("Server not running, no sessions found.");
                process::exit(0);
            }
        }
    };
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone UnixStream"));
    let json = serde_json::to_string(&request).expect("Failed to serialize request");

    writeln!(stream, "{}", json).expect("Failed to write to stream");

    reader
        .read_line(&mut resp_line)
        .expect("Failed to read response");

    // Debug: print what we received
    eprintln!("DEBUG: Received response line: '{}'", resp_line.trim());
    eprintln!("DEBUG: Response line length: {}", resp_line.len());

    if resp_line.trim().is_empty() {
        eprintln!("ERROR: Received empty response from server");
        process::exit(1);
    }

    let response: serde_json::Value = match serde_json::from_str(&resp_line) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to parse JSON response: {}", e);
            eprintln!("Raw response: '{}'", resp_line);
            process::exit(1);
        }
    };
    response
}

pub fn cleanup_sessions(store: &Arc<Mutex<Vec<Session>>>) -> Vec<Session> {
    let mut sessions = store.lock().unwrap();

    info!("Pruning Sessions: {:?}", sessions);

    sessions.iter_mut().for_each(|session| {
        session.jobs.retain(|job| is_job_suspended(job.pid));
    });

    sessions.retain(|session| !session.jobs.is_empty());

    info!("Updated Sessions: {:?}", sessions);

    sessions.to_vec()
}
