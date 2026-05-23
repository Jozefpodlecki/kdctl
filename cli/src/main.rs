use clap::Parser;
use flexi_logger::Logger;
use log::error;

use crate::{cli::{Cli, Commands}, commands::*};

mod cli;
mod commands;

fn main() {
    Logger::try_with_str("debug")
        .unwrap()
        .start()
        .unwrap();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install { start } => handle_install(start),
        Commands::Uninstall => handle_uninstall(),
        Commands::Status => handle_status(),
        Commands::Start => handle_start(),
        Commands::Stop => handle_stop(),
    };

    if let Err(err) = result {
        error!("{}", err);
    }
}