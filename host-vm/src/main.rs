use std::{ffi::OsString, time::Duration};
use flexi_logger::Logger;
use kdctl_client::KdctlClient;
use log::*;
use windows_service::{service::*, service_control_handler::ServiceControlHandlerResult, *};
use anyhow::{anyhow, Result};
use kdctl_shared::constants::*;

define_windows_service!(ffi_service_main, service_main);

fn service_main(arguments: Vec<OsString>) {
    if let Err(err) = run_service(arguments) {
        error!("Service error: {err}");
    }
}

fn run_service(arguments: Vec<OsString>) -> Result<()> {
    
    let addr = arguments[0].to_str().unwrap();
    let (client, signal) = KdctlClient::new();
    
    let event_handler = {
        move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                ServiceControl::Stop => {

                    signal.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }

                _ => ServiceControlHandlerResult::NotImplemented,
            }
        }
    };

    let status_handle = service_control_handler::register(CLIENT_SERVICE_NAME, event_handler)?;
    
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::from_bits_retain(SERVICE_TYPE),
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::from_bits_retain(SERVICE_TYPE),
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    client.run(addr)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::from_bits_retain(SERVICE_TYPE),
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

fn main() -> Result<(), windows_service::Error> {
    Logger::try_with_str("debug")
        .unwrap()
        .start()
        .unwrap();
    
    service_dispatcher::start(CLIENT_SERVICE_NAME, ffi_service_main)?;
    Ok(())
}