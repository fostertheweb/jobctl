use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
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
    pub cwd: PathBuf,
    pub number: u8,
    pub suspended: u64,
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
    List { jobs: HashMap<String, Vec<Job>> },
    Register { job: Job },
    Error { message: String },
}

pub fn encode_path(path: &PathBuf) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(path.to_string_lossy().as_bytes())
}

pub fn start_server() {
    let exe = env::current_exe().expect("Failed to get executable path");
    let server_path = exe.with_file_name("server");

    Command::new(server_path)
        .spawn()
        .expect("Failed to start server.");
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

    let response: serde_json::Value =
        serde_json::from_str(&resp_line).expect("Failed to parse JSON response");
    response
}

pub fn cleanup_sessions(
    store: &Arc<Mutex<HashMap<String, Vec<Job>>>>,
) -> std::sync::MutexGuard<'_, HashMap<String, Vec<Job>>> {
    let mut sessions = store.lock().unwrap();
    let keys = sessions.keys().cloned().collect::<Vec<String>>();

    for key in keys {
        let cwd = key.clone();
        if let Some(jobs) = sessions.get_mut(&key) {
            jobs.retain(|job| is_job_suspended(job.pid));

            if jobs.is_empty() {
                info!("Removing session for {}", cwd);
                sessions.remove(&cwd);
            }
        }
    }

    sessions
}
