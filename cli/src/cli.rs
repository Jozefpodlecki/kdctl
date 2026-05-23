use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kdctl")]
#[command(about = "Kernel driver controller", version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Install {
        #[arg(long)]
        start: bool,
    },
    Uninstall,
    Start,
    Stop,
    Status,
}