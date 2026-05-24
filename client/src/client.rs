use std::{
    sync::mpsc::{self, Sender, Receiver},
    time::Duration,
};
use anyhow::Result;
use kdctl_shared::{messages::{ClientMessage, ServerMessage}, stream::KdctlStream};
use log::{info, error};

use crate::retry::{RetryPolicy, RetryConfig};

pub struct KdctlClient {
    shutdown_tx: Sender<()>,
    shutdown_rx: Receiver<()>,
}

impl KdctlClient {
    pub fn new() -> (Self, Sender<()>) {
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let client = Self { shutdown_tx: shutdown_tx.clone(), shutdown_rx };
        (client, shutdown_tx)
    }

    pub fn run(&self, addr: &str) -> Result<()> {
        info!("Client connecting to {}", addr);
        let config = RetryConfig::default();
        let mut retry = RetryPolicy::new(config);

        loop {
            if self.should_shutdown()? {
                return Ok(());
            }

            match KdctlStream::connect(addr) {
                Ok(mut stream) => {
                    info!("Connected to host at {}", addr);
                    retry.reset();

                    match self.handle_connection(&mut stream) {
                        Ok(should_reconnect) => {
                            if !should_reconnect {
                                info!("Permanent disconnect (shutdown), exiting");
                                return Ok(());
                            }
                            info!("Disconnected from host, reconnecting...");
                            self.wait_with_backoff(&mut retry)?;
                        }
                        Err(e) => {
                            error!("Connection error: {}", e);
                            info!("Reconnecting...");
                            self.wait_with_backoff(&mut retry)?;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect: {}", e);
                    self.wait_with_backoff(&mut retry)?;
                }
            }
        }
    }

    fn handle_connection(&self, stream: &mut KdctlStream) -> Result<bool> {
        let ident = ClientMessage::Identify {
            client_name: "kdctl-client".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: vec![],
        };
        stream.send(&ident)?;
        info!("Sent identification");

        loop {
            let msg: ServerMessage = stream.recv()?;
            
            let response = self.handle_server_message(msg);
            
            match response {
                Some(resp) => {
                    stream.send(&resp)?;
                }
                None => {
                    info!("Shutdown signaled, exiting without reconnect");
                    return Ok(false);
                }
            }
        }
    }

    fn handle_server_message(&self, msg: ServerMessage) -> Option<ClientMessage> {
        match msg {
            // ServerMessage::Command { id, cmd, args } => {
            //     // let result = self.execute_command(&cmd, &args);
            //     // Some(ClientMessage::CommandResult { id, result })
            //     None
            // }
            ServerMessage::Shutdown => {
                info!("Server requested shutdown");
                self.shutdown_tx.send(()).unwrap();
                None
            }
            _ => {
                error!("Unexpected message: {:?}", msg);
                None
            }
        }
    }

    fn should_shutdown(&self) -> Result<bool> {
        match self.shutdown_rx.try_recv() {
            Ok(_) => {
                info!("Shutting down client");
                Ok(true)
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                info!("Shutdown channel disconnected");
                Ok(true)
            }
            Err(mpsc::TryRecvError::Empty) => Ok(false),
        }
    }

    fn wait_with_backoff(&self, retry: &mut RetryPolicy) -> Result<()> {
        if retry.should_retry() {
            info!("Retrying in {:?}... (attempt {})", retry.current_delay(), retry.attempt());
            retry.wait();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Max retries exceeded"))
        }
    }
}