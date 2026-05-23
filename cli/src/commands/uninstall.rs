use anyhow::{Result, bail};
use log::*;
use windows_service::service::{ServiceAccess, ServiceState};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use kdctl_shared::constants::*;

pub fn handle_uninstall() -> Result<()> {
    info!("Uninstalling {}", SERVICE_NAME);

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service = match service_manager.open_service(
        SERVICE_NAME,
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

    // let current_exe = std::env::current_exe()?;
    // let install_dir = current_exe.parent().unwrap();

    // let server_path = install_dir.join("kdctl-server.exe");
    // if server_path.exists() {
    //     std::fs::remove_file(&server_path)?;
    //     info!("Removed: {}", server_path.display());
    // }

    // let log_dir = std::path::Path::new(r"C:\ProgramData\kdctl\logs");
    // if log_dir.exists() && log_dir.read_dir()?.next().is_none() {
    //     std::fs::remove_dir(log_dir)?;
    //     info!("Removed empty log directory: {}", log_dir.display());
    // }

    // info!("Uninstall complete");
    
    Ok(())
}