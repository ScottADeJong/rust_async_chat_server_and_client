use std::fmt;

/// `ConfigError` is an enumeration that represents the possible errors that can occur
/// while working with configuration files or settings in an application.
///
/// # Variants
/// - `NoConfigOrFlag`
///   Indicates that neither a configuration file nor a flag with required settings is provided.
/// - `NoValidSettings`
///   Represents a case where no valid settings could be determined from the provided configuration.
/// - `ConfigReadFailed`
///   Returned when the configuration file could not be read, possibly due to file system issues
///   or incorrect file paths.
/// - `ConfigParseFailed`
///   Occurs when the configuration file is read but fails to parse, typically due to incorrect
///   formatting or invalid syntax.
/// - `MissingHostIp`
///   Signifies that a required field `HostIp` is missing in the configuration.
///
/// # Traits
/// - `Debug`
///   Allows the `ConfigError` enum to be formatted using the `{:?}` formatter, primarily for debugging purposes.
#[derive(Debug)]
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