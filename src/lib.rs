pub mod cli;
pub mod sessions;
pub mod utils;

#[derive(Debug)]
pub enum JobCtlError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Server(String),
}

#[derive(Debug)]
pub enum ClientError {
    Connection(std::io::Error),
    Serialization(serde_json::Error),
    ServerNotRunning,
    EmptyResponse,
    InvalidResponse(String),
    ServerError(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Connection(err) => write!(f, "Connection error: {}", err),
            ClientError::Serialization(err) => write!(f, "Serialization error: {}", err),
            ClientError::ServerNotRunning => write!(f, "Server not running, no sessions found"),
            ClientError::EmptyResponse => write!(f, "Received empty response from server"),
            ClientError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            ClientError::ServerError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        ClientError::Connection(err)
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(err: serde_json::Error) -> Self {
        ClientError::Serialization(err)
    }
}

