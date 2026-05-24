use anyhow::{Result, bail};
use serde::{Serialize, Deserialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KdctlConfig {
    pub host: HostConfig,
    pub client: ClientConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostConfig {
    pub listen_addr: String,
    pub service_name: String,
    pub install_dir: PathBuf,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientConfig {
    pub vm_host: String,
    pub service_name: String,
    pub install_dir: PathBuf,
    pub computer_name: String,
    pub username: String,
    pub password: String,
}

impl Default for KdctlConfig {
    fn default() -> Self {
        Self {
            host: HostConfig {
                listen_addr: "0.0.0.0:12345".to_string(),
                service_name: "kdctl-host".to_string(),
                install_dir: PathBuf::from(r"C:\Program Files\kdctl"),
            },
            client: ClientConfig {
                vm_host: "192.168.1.100:12345".to_string(),
                service_name: "kdctl-host-vm".to_string(),
                install_dir: PathBuf::from(r"C:\Program Files\kdctl"),
                computer_name: "192.168.91.128".to_string(),
                username: "".to_string(),
                password: "".to_string(),
            },
        }
    }
}

impl KdctlConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            bail!("Config not found. Run 'kdctl init' first");
        }
        
        let content = fs::read_to_string(&config_path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;

        Ok(())
    }

    pub fn save_default() -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let config = Self::default();
        config.save(&config_path)?;
        
        Ok(())
    }
    
    pub fn get_config_path() -> Result<PathBuf> {
        let current_exe = std::env::current_exe()?;
        let parent_dir = current_exe.parent().unwrap();

        // PathBuf::from(r"C:\Program Files\kdctl")
        let path = parent_dir
            .join("config.json");

        Ok(path)
    }
}