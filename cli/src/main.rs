use clap::Parser;
use flexi_logger::Logger;
use log::error;

use crate::{cli::{Cli, Commands}, commands::*};

mod cli;
mod commands;
mod config;

fn main() {
    Logger::try_with_str("debug")
        .unwrap()
        .start()
        .unwrap();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::InstallHost { start } => handle_install_host(start),
        Commands::UpdateHost => handle_update_host(),
        Commands::InstallClient { start } => handle_install_client(start),
        Commands::Uninstall => handle_uninstall(),
        Commands::Status => handle_host_status(),
        Commands::Start => handle_start(),
        Commands::Stop => handle_stop(),
        Commands::Init => handle_init(),
    };

    if let Err(err) = result {
        error!("Command failed: {}", err);
       for cause in err.chain().skip(1) {
    if let Some(service_err) = cause.downcast_ref::<windows_service::Error>() {
        match service_err {
            windows_service::Error::Winapi(error) => {
                error!("  Caused by: WinAPI error {}", error);
            }
            windows_service::Error::LaunchArgumentsNotSupported => {
                error!("  Caused by: Launch arguments not supported");
            }
            windows_service::Error::ParseValue(value, parse_error) => {
                error!("  Caused by: Failed to parse '{}': {}", value, parse_error);
            }
            windows_service::Error::ArgumentHasNulByte(arg) => {
                error!("  Caused by: Argument contains NUL byte: {:?}", arg);
            }
            windows_service::Error::ArgumentArrayElementHasNulByte(index, arg) => {
                error!("  Caused by: Argument[{}] contains NUL byte: {:?}", index, arg);
            }
            _ => {
                error!("  Caused by: {}", service_err);
            }
        }
    } else {
        error!("  Caused by: {}", cause);
    }
}
     
    }
}