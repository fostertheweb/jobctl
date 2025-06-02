use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{env, thread};

use crate::cli::Commands;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Session {
    id: u32,
    cwd: String,
    jobs: Vec<Job>,
    started: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Job {
    pub pid: u32,
    pub cmd: String,
    pub cwd: PathBuf,
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
    List { jobs: HashMap<String, Job> },
    Register { job: Job },
    Init { shell: String },
    Error { message: String },
}

pub fn encode_path(path: &Path) -> String {
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
                panic!("Server not running, no sessions found.");
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
