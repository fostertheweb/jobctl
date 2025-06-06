use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

pub const ZSH: &str = include_str!("../resources/hooks.zsh");

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Commands {
    List,
    Register {
        #[arg(short, long)]
        pid: u32,
        #[arg(short, long)]
        number: u8,
        #[arg(skip)]
        command: String,
    },
    Init {
        shell: String,
    },
}

#[derive(Parser)]
#[command(author, version, about)]
pub struct ServerArgs {
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
