use std::fs;
use std::thread::sleep;
use std::time::Duration;
use anyhow::{Result, bail};
use log::*;
use windows_service::service::*;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use kdctl_shared::constants::*;

use crate::config::KdctlConfig;

pub fn handle_update_host() -> Result<()> {
    info!("Updating host service...");
    let config = KdctlConfig::load()?;

    let service = open_service(HOST_SERVICE_NAME, ServiceAccess::STOP | ServiceAccess::QUERY_STATUS | ServiceAccess::START)?;
    
    stop_service(&service)?;
    
    copy_service_binary(&config)?;
    
    start_service(&service)?;

    info!("Host service update complete");
    Ok(())
}

fn open_service(name: &str, access: ServiceAccess) -> Result<windows_service::service::Service> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
    
    match service_manager.open_service(name, access) {
        Ok(s) => Ok(s),
        Err(e) => bail!("Service '{}' not found: {}", name, e),
    }
}

fn stop_service(service: &windows_service::service::Service) -> Result<()> {
    let status = service.query_status()?;
    
    if status.current_state != ServiceState::Running {
        info!("Service is not running (state: {:?})", status.current_state);
        return Ok(());
    }
    
    info!("Stopping service...");
    
    match service.stop() {
        Ok(_) => wait_for_state(service, ServiceState::Stopped, ServiceState::StopPending, "stop")?,
        Err(e) => {
            warn!("Graceful stop failed: {}. Attempting force stop...", e);
            force_stop_service()?;
            wait_for_state(service, ServiceState::Stopped, ServiceState::StopPending, "stop")?;
        }
    }
    
    info!("Service stopped");
    Ok(())
}

fn force_stop_service() -> Result<()> {
    let output = std::process::Command::new("sc")
        .args(&["stop", HOST_SERVICE_NAME])
        .output()?;
    
    if !output.status.success() {
        bail!("Force stop failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}

fn start_service(service: &windows_service::service::Service) -> Result<()> {
    info!("Starting service...");
    
    service.start(&Vec::<&str>::new())?;
    wait_for_state(service, ServiceState::Running, ServiceState::StartPending, "start")?;
    
    info!("Service started");
    Ok(())
}

fn wait_for_state(service: &windows_service::service::Service, target: ServiceState, pending: ServiceState, action: &str) -> Result<()> {
    let max_attempts = 30;
    
    for attempt in 0..max_attempts {
        let status = service.query_status()?;
        
        if status.current_state == target {
            return Ok(());
        }
        
        if status.current_state == pending {
            info!("Service is {}... (attempt {}/{})", action, attempt + 1, max_attempts);
        } else {
            info!("Service state: {:?}, waiting for {}...", status.current_state, action);
        }
        
        sleep(Duration::from_millis(500));
    }
    
    let final_status = service.query_status()?;
    bail!("Service failed to {} after {} seconds. Current state: {:?}", 
          action, max_attempts / 2, final_status.current_state);
}

fn copy_service_binary(config: &KdctlConfig) -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let file_name = format!("{}.exe", HOST_SERVICE_NAME);
    let source_path = current_exe.parent().unwrap().join(&file_name);
    
    if !source_path.exists() {
        bail!("{} not found at: {}", HOST_SERVICE_NAME, source_path.display());
    }
    
    let target_path = config.host.install_dir.join(file_name);
    
    info!("Copying new service binary...");
    fs::copy(source_path, &target_path)?;
    info!("Copied to: {}", target_path.display());
    
    Ok(())
}