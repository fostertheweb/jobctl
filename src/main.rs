use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

fn main() {
    // handle client connection
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::List) => {
            println!("Listing jobs...");
            // find jobs associated with current directory
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
}
