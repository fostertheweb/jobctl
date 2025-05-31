use crate::sessions::Job;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Commands {
    /// List all jobs
    List,
    Attach {
        /// Attach to a specific job
        // If attaching for the first time, register job with the server
        #[arg(short, long)]
        target: Option<u8>,
    },
    Kill {
        /// Kill a specific job
        #[arg(short, long)]
        target: u8,
    },
    Rename {
        /// Rename a specific job
        #[arg(short, long)]
        target: u8,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ServerResponse {
    List { jobs: Vec<Job> },
    Error { message: String },
}
