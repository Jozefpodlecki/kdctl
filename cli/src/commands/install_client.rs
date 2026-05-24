use anyhow::{Result, bail};
use log::*;
use windows_service::service::*;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use std::path::Path;
use std::process::Command;

use crate::config::KdctlConfig;

pub fn handle_install_client(start: bool) -> Result<()> {
    let config = KdctlConfig::load()?;
    
    info!("Installing client on remote VM: {}", config.client.vm_host);

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let host_service = match service_manager.open_service(&config.host.service_name, ServiceAccess::QUERY_STATUS) {
        Ok(s) => s,
        Err(_) => bail!("Host service '{}' is not installed on this machine", config.host.service_name),
    };

    let status = host_service.query_status()?;
    if status.current_state != ServiceState::Running {
        bail!("Host service is not running. Start it with 'kdctl start' first");
    }

    info!("Host service is running");

    let current_exe = std::env::current_exe()?;
    let client_exe_path = current_exe.parent().unwrap().join(&config.client.service_name);

    if !client_exe_path.exists() {
        bail!("{} not found at: {}", config.client.service_name, client_exe_path.display());
    }

    info!("Copying client to VM...");
    copy_to_vm(&client_exe_path, &config)?;

    info!("Installing client service on VM...");
    install_service_on_vm(&config, start)?;

    info!("Client installation complete");
    Ok(())
}

fn copy_to_vm(local_path: &Path, config: &KdctlConfig) -> Result<()> {
    let target_dir = &config.client.install_dir;
    let target_exe = target_dir.join(format!("{}.exe", config.client.service_name));
    
    let script = format!(
        r#"
        $password = ConvertTo-SecureString '{}' -AsPlainText -Force;
        $cred = New-Object System.Management.Automation.PSCredential('{}', $password)
        $session = New-PSSession -ComputerName '{}' -Credential $cred
        Copy-Item -Path '{}' -Destination '{}' -ToSession $session -Force
        Remove-PSSession $session
        "#,
        config.client.username,
        config.client.password,
        config.client.computer_name,
        local_path.display(),
        target_exe.display()
    );

    let output = Command::new("powershell")
        .args(&["-Command", &script])
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to copy client to VM: {}", err);
    }

    info!("Client copied to VM: {}", target_exe.display());
    Ok(())
}

fn install_service_on_vm(config: &KdctlConfig, start: bool) -> Result<()> {
    let service_name = &config.client.service_name;
    let exe_path = config.client.install_dir.join(format!("{}.exe", service_name));
    let vm_host = &config.client.computer_name;
    let username = &config.client.username;
    let password = &config.client.password;

    let start_service_cmd = if start {
        format!("Start-Service -Name '{}'", service_name)
    } else {
        String::new()
    };

    let script = format!(
        r#"
        $password = ConvertTo-SecureString '{}' -AsPlainText -Force;
        $cred = New-Object System.Management.Automation.PSCredential('{}', $password);
        $session = New-PSSession -ComputerName '{}' -Credential $cred
        Invoke-Command -Session $session -ScriptBlock {{
            $serviceName = '{}'
            $service = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
            if ($service) {{
                Stop-Service -Name $serviceName -Force
                sc.exe delete $serviceName
                Start-Sleep -Seconds 2
            }}
            New-Service -Name $serviceName `
                -BinaryPathName '{}' `
                -DisplayName '{}' `
                -Description '{}' `
                -StartupType Automatic
            {}
        }}
        Remove-PSSession $session
        "#,
        username,
        password,
        vm_host,
        service_name,
        exe_path.display(),
        format!("KDCTL Client Service"),
        format!("Executes driver operations on VM"),
        start_service_cmd
    );

    let output = Command::new("powershell")
        .args(&["-Command", &script])
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to install service on VM: {}", err);
    }

    info!("Service installed on VM");
    Ok(())
}