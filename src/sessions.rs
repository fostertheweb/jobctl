use serde::{Deserialize, Serialize};

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
