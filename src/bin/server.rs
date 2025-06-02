use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
};

use jobctl::cli::Commands;
use jobctl::sessions::{ClientRequest, Job, ServerResponse};

fn handle_client(mut stream: UnixStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    reader.read_line(&mut line)?;

    let req: ClientRequest = match serde_json::from_str(&line) {
        Ok(r) => {
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            r
        }
        Err(e) => {
            let err = ServerResponse::Error {
                message: format!("invalid request: {}", e),
            };
            let payload = serde_json::to_string(&err).unwrap();
            writeln!(stream, "{}", payload)?;
            return Ok(());
        }
    };

    let response = match req.action {
        Commands::List => ServerResponse::List {
            jobs: HashMap::new(),
        },
        Commands::Register {
            pid,
            command,
            suspended,
        } => {
            let job = Job {
                suspended,
                cwd: req.cwd,
                pid: pid,
                cmd: command,
            };

            ServerResponse::Register { job }
        }
        Commands::Init { shell } => ServerResponse::Init { shell },
    };

    let payload = serde_json::to_string(&response).unwrap();
    writeln!(stream, "{}", payload)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let uid = unsafe { libc::getuid() };
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
