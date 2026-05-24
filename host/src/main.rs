use std::{ffi::OsString, thread::sleep, time::Duration};
use flexi_logger::{Criterion, FileSpec, Logger, WriteMode};
use log::*;
use kdctl_server::KdctlServer;
use windows_service::{service::*, service_control_handler::ServiceControlHandlerResult, *};
use anyhow::{Result, anyhow};
use kdctl_shared::constants::*;

use crate::event_log::EventLogWriter;

mod event_log;

define_windows_service!(ffi_service_main, service_main);

fn service_main(arguments: Vec<OsString>) {
    if let Err(err) = run_service(arguments) {
        error!("Service fatal error: {:#}", err);
    }
}

fn init_logger() -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let parent_dir = current_exe.parent().unwrap();
    let event_writer = Box::new(EventLogWriter::new("KDCTL Server"));

    Logger::try_with_str("debug")
        .map_err(|e| anyhow!("Failed to set log level: {}", e))?
        .log_to_file(
            FileSpec::default()
                .directory(parent_dir)
                .basename("kdctl-host")
        )
        .write_mode(WriteMode::BufferAndFlush)
        .add_writer("EventLog", event_writer)
        .start()
        .map_err(|e| anyhow!("Failed to start logger: {}", e))?;
    
    Ok(())
}

fn run_service(arguments: Vec<OsString>) -> Result<()> {
    init_logger()?;

    if arguments.is_empty() {
        error!("No address argument provided");
        return Err(anyhow!("Missing address argument"));
    }

    info!("Arguments: {:?}", arguments);
    
    let addr = arguments[1].to_str()
        .ok_or_else(|| anyhow!("Invalid address argument"))?;
    
    info!("Starting service on {}", addr);
    
    let (server, signal) = KdctlServer::new();
    
    let event_handler = {
        let signal = signal.clone();
        move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                ServiceControl::Stop => {
                    let _ = signal.send(());
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        }
    };

    let status_handle = service_control_handler::register(HOST_SERVICE_NAME, event_handler)
        .map_err(|e| anyhow!("Failed to register handler: {}", e))?;
    
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::from_bits_retain(SERVICE_TYPE),
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(30),
        process_id: None,
    })?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::from_bits_retain(SERVICE_TYPE),
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(30),
        process_id: None,
    })?;

    info!("Socket addr: {}", addr);
    if let Err(e) = server.run(addr) {
        error!("Server failed: {:#}", e);
        let _ = status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::from_bits_retain(SERVICE_TYPE),
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(12345),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        });
        log::logger().flush();
        return Err(e);
    }

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::from_bits_retain(SERVICE_TYPE),
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(123),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

fn main() -> Result<(), windows_service::Error> {
    service_dispatcher::start(HOST_SERVICE_NAME, ffi_service_main)?;
    Ok(())
}