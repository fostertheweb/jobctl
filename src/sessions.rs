use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{env, thread};

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
    pub session: u32,
    pub cmd: Vec<String>,
    pub cwd: String,
    pub name: String,
    pub started: u64,
}

pub fn encode_path(path: &PathBuf) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(path.to_string_lossy().as_bytes())
}

pub fn connect(socket_path: &Path) -> UnixStream {
    match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(_) => {
            let exe = env::current_exe().expect("Failed to get executable path");
            let server_path = exe.with_file_name("server");

            Command::new(server_path)
                .spawn()
                .expect("Failed to start server");
            thread::sleep(Duration::from_millis(500));

            UnixStream::connect(&socket_path).expect("Failed to connect to server socket")
        }
    }
}
