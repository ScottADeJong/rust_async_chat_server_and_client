use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ConfigError {
    NoConfigOrFlag,
    NoValidSettings,
    ConfigReadFailed,
    ConfigParseFailed,
    MissingHostIp
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::NoConfigOrFlag => write!(f, "No config file or flag provided."),
            ConfigError::NoValidSettings => write!(f, "No valid settings in provided config file. Review Template."),
            ConfigError::ConfigReadFailed => write!(f, "Failed to read the config file, do you have permissions?"),
            ConfigError::ConfigParseFailed => write!(f, "Failed to parse the config file, is it valid?"),
            ConfigError::MissingHostIp => write!(f, "Missing host IP in the config file.")
        }
    }
}