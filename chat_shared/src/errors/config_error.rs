use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    NoConfigOrFlag,
    NoValidSettings,
    ConfigReadFailed(String),
    MissingKeys(Vec<String>)
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::NoConfigOrFlag => write!(f, "No config file or flag provided."),
            ConfigError::NoValidSettings => write!(f, "No valid settings in provided config file. Review Template."),
            ConfigError::ConfigReadFailed(e) => write!(f, "Config file read failed: {}", e),
            ConfigError::MissingKeys(keys) => write!(f, "Missing keys: {:?}", keys)
        }
    }
}