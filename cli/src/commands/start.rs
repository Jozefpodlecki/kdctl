use anyhow::{Result, bail};
use log::info;
use windows_service::service::{ServiceAccess, ServiceState};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use kdctl_shared::constants::*;

pub fn handle_start() -> Result<()> {
    info!("Starting {}...", SERVICE_NAME);

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service = match service_manager.open_service(SERVICE_NAME, ServiceAccess::START | ServiceAccess::QUERY_STATUS) {
        Ok(s) => s,
        Err(e) => bail!("Service not found: {}", e),
    };

    let status = service.query_status()?;
    
    if status.current_state == ServiceState::Running {
        info!("Service already running");
        return Ok(());
    }

    service.start(&Vec::<&str>::new())?;
    
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let status = service.query_status()?;
        if status.current_state == ServiceState::Running {
            info!("Service started successfully");
            return Ok(());
        }
    }
    
    bail!("Service failed to start within timeout");
}