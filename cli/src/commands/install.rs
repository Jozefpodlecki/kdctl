use std::ffi::OsString;
use anyhow::{Result, bail};
use kdctl_shared::wait_for_server;
use log::*;
use windows_service::service::*;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use kdctl_shared::constants::*;

pub fn handle_install(start: bool) -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let server_path = current_exe
        .parent()
        .unwrap()
        .join(format!("{}.exe", SERVICE_NAME));

    if !server_path.exists() {
        bail!("{} not found at: {}", SERVICE_NAME, server_path.display());
    }

    debug!("Accessing Service Manager...");

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    match service_manager.open_service(SERVICE_NAME, ServiceAccess::ALL_ACCESS) {
        Ok(_) => bail!("Service already exists"),
        Err(_) => {},
    }

    let service_binary_path = ::std::env::current_exe()
        .unwrap()
        .with_file_name(SERVICE_NAME);

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from("KDCTL Server"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    debug!("Installing kdctl-server as Windows service...");

    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description("Kernel driver controller server - listens for commands from CLI and forwards to VM client")?;

    debug!("Service installed successfully");

    if start {
        service.start(&Vec::<&str>::new())?;
        wait_for_server()?;
    }

    Ok(())
}