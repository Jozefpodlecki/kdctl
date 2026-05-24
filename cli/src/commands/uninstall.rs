use anyhow::{Result, bail};
use log::*;
use windows_service::service::{ServiceAccess, ServiceState};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use kdctl_shared::constants::*;
use std::fs;

use crate::config::KdctlConfig;

pub fn handle_uninstall() -> Result<()> {
    let config = KdctlConfig::load()?;

    info!("Uninstalling {}", HOST_SERVICE_NAME);

    remove_service(HOST_SERVICE_NAME)?;
    remove_directory(&config.host.install_dir)?;

    info!("Uninstall complete");
    
    Ok(())
}

fn remove_service(service_name: &str) -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service = match service_manager.open_service(
        service_name,
        ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE
    ) {
        Ok(s) => s,
        Err(e) => bail!("Service not found or cannot be opened: {}", e),
    };

    let status = service.query_status()?;

    if status.current_state == ServiceState::Running {
        info!("Stopping service...");
        service.stop()?;

        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let status = service.query_status()?;
            if status.current_state == ServiceState::Stopped {
                info!("Service stopped");
                break;
            }
        }
    }

    info!("Deleting service...");
    service.delete()?;
    info!("Service deleted");

    Ok(())
}

fn remove_directory(dir: &std::path::Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    info!("Cleaning up directory: {}", dir.display());

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            fs::remove_file(&path)?;
            info!("Removed file: {}", path.display());
        }
    }

    fs::remove_dir(dir)?;
    info!("Removed directory: {}", dir.display());

    Ok(())
}