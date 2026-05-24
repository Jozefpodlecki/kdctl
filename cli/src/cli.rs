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
    Init,
    
    /// Installs the host service on the local machine
    /// The host service listens for commands from the CLI and forwards them to the VM client
    InstallHost {
        /// Automatically start the service after installation
        #[arg(long)]
        start: bool,
    },

    UpdateHost,

    /// Installs the client service on the target VM
    /// The client service runs on the VM and executes operations
    InstallClient {
        /// Automatically start the service after installation
        #[arg(long)]
        start: bool,
    },

    /// Uninstalls both host and client services
    /// Removes service registration and binary files
    Uninstall,

    /// Starts the host service
    /// The service will begin accepting TCP connections
    Start,

    /// Stops the host service
    Stop,

    /// Shows status of host service and TCP connectivity
    Status,
}