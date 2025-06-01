use clap::Parser;
use jobctl::cli::{Cli, Commands};
use jobctl::sessions::{connect, encode_path};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir().expect("Failed to get current directory");
    let uid = unsafe { libc::getuid() };
    let socket_path = format!("/tmp/jobctl-{}.sock", uid);
    let _session_id = encode_path(&cwd);

    let mut resp_line = String::new();

    match &cli.command {
        Some(Commands::List) => {
            let mut stream = connect(Path::new(&socket_path));
            let mut reader =
                BufReader::new(stream.try_clone().expect("Failed to clone UnixStream"));
            let json = serde_json::to_string(&cli.command).unwrap();

            writeln!(stream, "{}", json)?;

            reader
                .read_line(&mut resp_line)
                .expect("Failed to read response");

            let response: serde_json::Value =
                serde_json::from_str(&resp_line).expect("Failed to parse JSON response");

            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Some(Commands::Attach { target }) => {
            match target {
                Some(target) => {
                    println!("Attaching to {}", target);
                    // attach to background job via PID
                }
                None => {
                    println!("Registering job with server");
                    // register background job with server
                }
            }
        }
        Some(Commands::Kill { target }) => {
            println!("Killing {}", target);
            // kill background job via PID
        }
        Some(Commands::Rename { target }) => {
            println!("Rename {}", target);
            // rename background job via PID
        }
        None => {}
    }

    Ok(())
}
