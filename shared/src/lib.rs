pub mod commands;
pub mod constants;

use std::{net::TcpStream, time::Duration};

use anyhow::{Result, anyhow};
use log::*;

pub fn wait_for_server() -> Result<()> {
    let mut retries = 10;
    while retries > 0 {

        std::thread::sleep(Duration::from_millis(500));
        if TcpStream::connect("127.0.0.1:12345").is_ok() {
            info!("TCP server is ready");
            return Ok(());
        }
        retries -= 1;
    }
    Ok(())
}