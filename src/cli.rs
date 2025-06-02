use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

pub const ZSH: &str = include_str!("../resources/hooks.zsh");

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Commands {
    List,
    Register {
        pid: u32,
        #[arg(skip)]
        command: String,
        #[arg(skip)]
        suspended: u64,
    },
    Init {
        shell: String,
    },
}
