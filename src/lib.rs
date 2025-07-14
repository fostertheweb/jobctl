pub mod cli;
pub mod sessions;
pub mod utils;

#[derive(Debug)]
pub enum JobCtlError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Server(String),
}

