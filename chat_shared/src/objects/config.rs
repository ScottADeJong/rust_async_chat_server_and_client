use std::fs::File;
use std::io::Read;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;
use crate::ConfigError;
use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG_FILE: &str = "../env/config.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub host_ipv4: Option<Ipv4Addr>,
    pub host_ipv6: Option<Ipv6Addr>,
    pub host_port: usize,
    pub msg_size: u8,
    pub prefix: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host_ipv4: Some(Ipv4Addr::from([127, 0, 0, 1])),
            host_ipv6: None,
            host_port: 7070,
            msg_size: 255,
            prefix: ":".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file(config_file: Option<&Path>) -> Result<Self, ConfigError> {

        let config_file = config_file.unwrap_or_else(|| {
            Path::new(DEFAULT_CONFIG_FILE)
        });

        let mut file = File::open(config_file).map_err(|_| ConfigError::NoConfigOrFlag)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents).map_err(|_| ConfigError::ConfigReadFailed)?;

        ron::from_str(&contents).map_err(|_| ConfigError::ConfigParseFailed)?
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.host_ipv4.is_none() && self.host_ipv6.is_none() {
            return Err(ConfigError::MissingHostIp)
        }
        
        Ok(())
    }
    
    pub fn get_ip(&self) -> String {
        match self.host_ipv4 {
            Some(ip) => ip.to_string(),
            None => self.host_ipv6.as_ref().unwrap().to_string(),
        }
    }
}