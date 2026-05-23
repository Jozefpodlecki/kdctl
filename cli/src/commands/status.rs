use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use log::{info, warn};
use windows_service::service::ServiceState;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use kdctl_shared::constants::*;

pub fn handle_status() -> anyhow::Result<()> {
    let service_running = check_service_status()?;
    
    if service_running {
        check_server()?;
    } else {
        warn!("Skipping TCP check because service is not running");
    }
    
    Ok(())
}

fn check_service_status() -> anyhow::Result<bool> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service = match service_manager.open_service(SERVICE_NAME, windows_service::service::ServiceAccess::QUERY_STATUS) {
        Ok(s) => s,
        Err(_) => {
            info!("Service: NOT INSTALLED");
            return Ok(false);
        }
    };

    let status = service.query_status()?;
    
    match status.current_state {
        ServiceState::Running => {
            info!("Service: RUNNING");
            Ok(true)
        }
        ServiceState::Stopped => {
            info!("Service: STOPPED");
            Ok(false)
        }
        ServiceState::StartPending => {
            info!("Service: START_PENDING");
            Ok(false)
        }
        ServiceState::StopPending => {
            info!("Service: STOP_PENDING");
            Ok(false)
        }
        ServiceState::ContinuePending => {
            info!("Service: CONTINUE_PENDING");
            Ok(false)
        }
        ServiceState::PausePending => {
            info!("Service: PAUSE_PENDING");
            Ok(false)
        }
        ServiceState::Paused => {
            info!("Service: PAUSED");
            Ok(false)
        }
        _ => {
            info!("Service: UNKNOWN");
            Ok(false)
        }
    }
}

fn check_server() -> anyhow::Result<()> {
    match TcpStream::connect_timeout(&"127.0.0.1:12345".parse()?, Duration::from_secs(2)) {
        Ok(mut stream) => {
            let ping = serde_json::json!({"cmd": "ping"});
            stream.write_all(ping.to_string().as_bytes())?;
            
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf)?;
            let response = String::from_utf8_lossy(&buf[..n]);
            info!("TCP server: RESPONDED ({})", response);
        }
        Err(e) => {
            warn!("TCP server: NOT RESPONDING - {}", e);
        }
    }
    Ok(())
}