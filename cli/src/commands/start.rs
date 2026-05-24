use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use log::info;
use windows_service::service::{ServiceAccess, ServiceState};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use kdctl_shared::constants::*;

pub fn handle_start() -> Result<()> {
    
    let host = "0.0.0.0";
    let port = 12345;
    let addr = format!("{}:{}", host, port);

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service = match service_manager.open_service(HOST_SERVICE_NAME, ServiceAccess::START | ServiceAccess::QUERY_STATUS) {
        Ok(s) => s,
        Err(e) => bail!("Service not found: {}", e),
    };

    info!("Querying service...");
    let status = service.query_status().with_context(|| "Could not query service")?;
    
    if status.current_state == ServiceState::Running {
        info!("Service already running");
        return Ok(());
    }

    info!("Starting {}...", HOST_SERVICE_NAME);
    service.start(&vec![&addr]).with_context(|| "Could not start service")?;
    
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(500));
        let status = service.query_status().with_context(|| "Could not query service")?;
        if status.current_state == ServiceState::Running {
            info!("Service started successfully");
            return Ok(());
        }
    }
    
    bail!("Service failed to start within timeout");
}