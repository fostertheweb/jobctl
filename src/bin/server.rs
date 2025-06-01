use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use jobctl::cli::{Commands, ServerResponse};
use jobctl::sessions::Job;

fn handle_client(mut stream: UnixStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    let mut jobs = HashMap::new();

    let job = Job {
        pid: 42,
        session: 0,
        cmd: vec!["neovim".to_string(), "README.md".to_string()],
        cwd: String::from("/Users/jonathan/.dotfiles"),
        name: String::from("editor"),
        started: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    jobs.insert(String::from("editor"), job);

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

    let response = match req {
        Commands::List => ServerResponse::List { jobs },
        Commands::Attach { target } => todo!(),
        Commands::Kill { target } => todo!(),
        Commands::Rename { target } => todo!(),
    };

    let payload = serde_json::to_string(&response).unwrap();
    writeln!(stream, "{}", payload)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let uid = unsafe { libc::getuid() };
    // consider creating jobctl-{uid} directory first with dfeault.sock
    let socket_path = format!("/tmp/jobctl-{}.sock", uid);
    if let Err(e) = fs::remove_file(PathBuf::from(&socket_path)) {
        match e.kind() {
            std::io::ErrorKind::NotFound => {}
            _ => {
                eprintln!("Failed to remove `{}`: {}", &socket_path, e)
            }
        }
    }
    let listener = UnixListener::bind(&socket_path)?;

    println!("Server listening on {}", socket_path);

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
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

    Ok(())
}
