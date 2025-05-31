use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use jobctl::cli::{Commands, ServerResponse};
use jobctl::sessions::Job;
use serde_json;

fn handle_client(mut stream: UnixStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    // Read exactly one line (i.e. one JSON message, newline‐delimited)
    reader.read_line(&mut line)?;
    let req: Commands = match serde_json::from_str(&line) {
        Ok(r) => r,
        Err(e) => {
            let err = ServerResponse::Error {
                message: format!("invalid request: {}", e),
            };
            let payload = serde_json::to_string(&err).unwrap();
            writeln!(stream, "{}", payload)?;
            return Ok(());
        }
    };

    // Dummy in-memory job table (for demo). In real use, you'd keep a `Vec<Job>` or `HashMap<...>`.
    // Here we pretend there's one job with pid=42
    let response = match req {
        Commands::List => {
            let jobs = vec![Job {
                pid: 1,
                session: 1,
                cmd: vec!["neovim".to_string(), "index.html".to_string()],
                cwd: String::from("/Users/jonathan/Developer/Source/jobctl"),
                name: String::from("editor"),
                started: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }];
            ServerResponse::List { jobs }
        }
        Commands::Attach { target } => todo!(),
        Commands::Kill { target } => todo!(),
        Commands::Rename { target } => todo!(),
    };

    // Serialize and send response (newline‐delimited)
    let payload = serde_json::to_string(&response).unwrap();
    writeln!(stream, "{}", payload)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let socket_path = "/tmp/jobctl.sock";
    // Remove old socket if present
    let _ = fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;
    println!("➜ Server listening on {}", socket_path);

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                // Spawn a thread to handle each client so the listener can keep accepting
                thread::spawn(|| {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Incoming error: {}", e);
                break;
            }
        }
    }

    // On graceful shutdown you might unlink the socket_path here
    Ok(())
}
