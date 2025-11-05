use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::{Ipv4Addr, Ipv6Addr};
use crate::ConfigError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub host_ipv4: Option<Ipv4Addr>,
    pub host_ipv6: Option<Ipv6Addr>,
    pub host_port: usize,
    pub msg_size: usize,
    pub prefix: String,
}

const DEFAULT_CONFIG_FILE: &str = "../env/config.toml";
const DEFAULT_CONFIG_KEYS: [&str; 4] = ["host_ip", "host_port", "msg_size", "prefix"];

pub struct ConfigHandle {
    pub options: HashMap<String, String>
}

fn parse_config(mut config_file: File) -> Result<HashMap<String, String>, ConfigError> {
    let mut options = HashMap::new();
    let mut config_file_string: String = String::new();

    if let Err(e) = config_file.read_to_string(&mut config_file_string) {
        return Err(ConfigError::ConfigReadFailed(e.to_string()))
    }

    for line in config_file_string.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() || (line.starts_with('[') && line.ends_with(']')) {
            continue;
        }

        let mut split_line = line.splitn(2, '=');
        if let (Some(key), Some(value)) = (split_line.next(), split_line.next()) {
            options.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    Ok(options)
}

impl ConfigHandle {
    pub fn new(config_file: Option<File>) -> Result<Self, ConfigError> {
        let file = match config_file {
            Some(file) => file,
            None => File::open(DEFAULT_CONFIG_FILE).map_err(|_| ConfigError::NoConfigOrFlag)?
        };

        let options = parse_config(file)?;
        if options.is_empty() {
            return Err(ConfigError::NoValidSettings)
        }

        let missing: Vec<String> = DEFAULT_CONFIG_KEYS
            .iter()
            .filter(|key| !options.contains_key(**key))
            .map(|s| s.to_string())
            .collect();

        if !missing.is_empty() {
            return Err(ConfigError::MissingKeys(missing))
        }

        Ok(Self { options})
    }

    pub fn get_value_string(&self, key: &str) -> Option<String> {
        self.options.get(key).map(|s| s.to_string())
    }

    pub fn get_value_usize(&self, key: &str) -> Option<usize> {
        self.options.get(key).map(|s| s.to_string().parse::<usize>().expect("Failed to convert value to a usize"))
    }
}