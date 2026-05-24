use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use log::{info, warn};
use windows_service::service::ServiceState;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use kdctl_shared::constants::*;

pub fn handle_host_status() -> anyhow::Result<()> {
    let service_running = check_service_status()?;
    
    if service_running {
        let host = "0.0.0.0";
        let port = 12345;
        let addr = format!("{}:{}", host, port);
        check_server(&addr)?;
    } else {
        warn!("Skipping TCP check because service is not running");
    }
    
    Ok(())
}

fn check_service_status() -> anyhow::Result<bool> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service = match service_manager.open_service(HOST_SERVICE_NAME, windows_service::service::ServiceAccess::QUERY_STATUS) {
        Ok(s) => s,
        Err(_) => {
            info!("Service is not installed");
            return Ok(false);
        }
    };

    let status = service.query_status()?;
    
    match status.current_state {
        ServiceState::Running => {
            info!("Service: RUNNING {:?}", status.exit_code);
            Ok(true)
        }
        ServiceState::Stopped => {
            info!("Service: STOPPED {:?}", status.exit_code);
            Ok(false)
        }
        ServiceState::StartPending => {
            info!("Service: START_PENDING {:?}", status.exit_code);
            Ok(false)
        }
        ServiceState::StopPending => {
            info!("Service: STOP_PENDING {:?}", status.exit_code);
            Ok(false)
        }
        ServiceState::ContinuePending => {
            info!("Service: CONTINUE_PENDING {:?}", status.exit_code);
            Ok(false)
        }
        ServiceState::PausePending => {
            info!("Service: PAUSE_PENDING {:?}", status.exit_code);
            Ok(false)
        }
        ServiceState::Paused => {
            info!("Service: PAUSED {:?}", status.exit_code);
            Ok(false)
        }
        _ => {
            info!("Service: UNKNOWN {:?}", status.exit_code);
            Ok(false)
        }
    }
}

fn check_server(addr: &str) -> anyhow::Result<()> {
    match TcpStream::connect_timeout(&addr.parse::<std::net::SocketAddr>()?, Duration::from_secs(2)) {
        Ok(mut stream) => {
            // let ping = serde_json::json!({"cmd": "ping"});
            // stream.write_all(ping.to_string().as_bytes())?;
            
            // let mut buf = [0u8; 1024];
            // let n = stream.read(&mut buf)?;
            // let response = String::from_utf8_lossy(&buf[..n]);
            info!("TCP server");
        }
        Err(e) => {
            warn!("TCP server: NOT RESPONDING - {}", e);
        }
    }
    Ok(())
}