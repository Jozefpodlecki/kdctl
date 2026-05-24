use anyhow::Result;
use log::info;
use crate::config::KdctlConfig;

pub fn handle_init() -> Result<()> {
    info!("Creating config template...");
    
    KdctlConfig::save_default()?;
    
    info!("Config created at: {}", KdctlConfig::get_config_path().unwrap().display());
    Ok(())
}