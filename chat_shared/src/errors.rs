use std::fmt;

#[derive(Debug)]
pub enum CliError {
    NoConfigOrFlag,
    NoValidSettings,
    ConfigReadFailed(String),
    MissingKeys(Vec<String>)
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::NoConfigOrFlag => write!(f, "No config file or flag provided."),
            CliError::NoValidSettings => write!(f, "No valid settings in provided config file. Review Template."),
            CliError::ConfigReadFailed(e) => write!(f, "Config file read failed: {}", e),
            CliError::MissingKeys(keys) => write!(f, "Missing keys: {:?}", keys)
        }
    }
}