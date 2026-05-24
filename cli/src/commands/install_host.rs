use std::ffi::OsString;
use std::fs;
use anyhow::{Result, bail};
use kdctl_shared::wait_for_server;
use log::*;
use windows_service::service::*;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use kdctl_shared::constants::*;

use crate::config::KdctlConfig;

pub fn handle_install_host(start: bool) -> Result<()> {
   
    let config = KdctlConfig::load()?;

    let current_exe = std::env::current_exe()?;
    let file_name = format!("{}.exe", HOST_SERVICE_NAME);
    let host_service_path = current_exe
        .parent()
        .unwrap()
        .join(&file_name);

    if !host_service_path.exists() {
        bail!("{} not found at: {}", HOST_SERVICE_NAME, host_service_path.display());
    }

    fs::create_dir_all(&config.host.install_dir)?;

    let config_path = config.host.install_dir.join("config.json");
    config.save(&config_path)?;
    
    let target_path = config.host.install_dir.join(file_name);
    fs::copy(host_service_path, &target_path)?;

    debug!("Accessing Service Manager...");

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    match service_manager.open_service(HOST_SERVICE_NAME, ServiceAccess::ALL_ACCESS) {
        Ok(_) => bail!("Service already exists"),
        Err(_) => {},
    }

    let service_info = ServiceInfo {
        name: OsString::from(HOST_SERVICE_NAME),
        display_name: OsString::from("KDCTL Server"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: target_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    debug!("Installing {HOST_SERVICE_NAME} as Windows service...");

    let service = service_manager.create_service(&service_info, ServiceAccess::ALL_ACCESS)?;
    service.set_description(HOST_SERVICE_DESCRIPTION)?;

    debug!("Service installed successfully");

    if start {
        debug!("Starting service...");
        service.start(&vec![&config.host.listen_addr])?;
        wait_for_server(&config.host.listen_addr)?;
    }

    Ok(())
}