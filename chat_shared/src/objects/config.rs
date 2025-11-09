use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use ron::ser::PrettyConfig;
use crate::ConfigError;
use serde::{Deserialize, Serialize};

/// A constant string slice that defines the default configuration file path.
///
/// `DEFAULT_CONFIG_FILE` specifies the location of the default configuration file
/// used by the application. The file is expected to be in the RON (Rusty Object Notation)
/// format and located at `env/config.ron`.
///
/// # Usage
///
/// This constant can be used whenever the application requires the default configuration file path:
///
/// ```rust
/// let config_path = DEFAULT_CONFIG_FILE;
/// println!("Loading configuration from: {}", config_path);
/// ```
///
/// # Notes
/// Ensure that the file exists and is properly formatted to prevent runtime errors during configuration loading.
const DEFAULT_CONFIG_FILE: &str = "env/config.ron";

/// The `Config` struct is used to define the configuration settings for a specific application or functionality.
///
/// This struct supports serialization and deserialization using Serde, and includes the following fields:
///
/// ## Fields
/// - `host_ipv4` (*`Option<Ipv4Addr>`*):
///   An optional IPv4 address specifying the host IP.
///   If `None`, no IPv4 address is configured.
/// - `host_ipv6` (*`Option<Ipv6Addr>`*):
///   An optional IPv6 address specifying the host IP.
///   If `None`, no IPv6 address is configured.
/// - `host_port` (*usize*):
///   The port number on which the host operates.
///   This is required and must be a valid TCP/UDP port.
/// - `msg_size` (*u8*):
///   Specifies the size of the message in bytes.
///   Represents the configured message buffer size.
/// - `prefix` (*char*):
///   A character used as a prefix within the application.
///   This may be used for message parsing or other internal purposes.
///
/// ## Derives
/// - `Serialize`: Allows the `Config` struct to be serialized into RON.
/// - `Deserialize`: Allows the `Config` struct to be deserialized from RON.
/// - `Debug`: Enables debug formatting for easy output during development.
///
/// ## Example Usage
/// ```rust
/// use std::net::{Ipv4Addr, Ipv6Addr};
/// use serde::{Serialize, Deserialize};
///
/// let config = Config {
///     host_ipv4: Some(Ipv4Addr::new(192, 168, 0, 1)),
///     host_ipv6: None,
///     host_port: 8080,
///     msg_size: 64,
///     prefix: '#',
/// };
///
/// println!("{:?}", config);
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub host_ipv4: Option<Ipv4Addr>,
    pub host_ipv6: Option<Ipv6Addr>,
    pub host_port: usize,
    pub msg_size: u8,
    pub prefix: char,
}

/// The `Default` trait is used to define a default configuration.
impl Default for Config {
    /// Provides a default implementation for the struct it is implemented for.
    ///
    /// # Default Values
    /// - `host_ipv4`: Set to `Some(127.0.0.1)`, which is the default loopback address for IPv4.
    /// - `host_ipv6`: Set to `None`, indicating no IPv6 address by default.
    /// - `host_port`: Set to `7070`, representing the default port to use.
    /// - `msg_size`: Set to `255`, defining the maximum message size.
    /// - `prefix`: Set to the character `:`. This is parsed from a string representation.
    ///
    /// # Panics
    /// This function will panic if the `char::from_str` for `:` fails, although such a failure
    /// is highly unlikely as `:` is a valid character.
    ///
    /// # Returns
    /// A new instance of the implementing struct with the pre-defined default values.
    ///
    /// # Usage
    /// ```rust
    /// use chat_shared::Config;
    /// let default_instance = Config::default();
    /// ```
    fn default() -> Self {
        Self {
            host_ipv4: Some(Ipv4Addr::from([127, 0, 0, 1])),
            host_ipv6: None,
            host_port: 7070,
            msg_size: 255,
            prefix: char::from_str(":").expect("':' COULD NOT CONVERT TO CHAR"),
        }
    }
}

/// Creates a default configuration file at the specified path and returns the default `Config` object.
///
/// This function performs the following steps:
/// 1. Attempts to create a file at the provided `path`.
/// 2. If file creation is successful:
///    - Generates a default `Config` object using `Config::default()`.
///    - Serializes the default configuration into a pretty RON (Rusty Object Notation) string using a specified format.
///    - Writes the serialized string to the newly created file.
///    - Returns the default `Config` object upon success.
/// 3. If file creation fails or any of the above operations encounter an error,
///    it returns a `ConfigError::NoConfigOrFlag`.
///
/// # Arguments
/// * `path` - A reference to a `PathBuf` that specifies the location to create the configuration file.
///
/// # Returns
/// * `Ok(Config)` - The successfully created default `Config`.
/// * `Err(ConfigError::NoConfigOrFlag)` - If an error occurs during file creation,
///   serialization, or writing to the file.
///
/// # Errors
/// * Returns `Err(ConfigError::NoConfigOrFlag)` if:
///   - The function fails to create the file at the specified path.
///   - There is an error serializing the default `Config` object to a RON string.
///   - There is an error writing the serialized data to the file.
///
/// # Examples
/// ```rust
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("config.ron");
/// match create_default(&path) {
///     Ok(config) => {
///         println!("Default configuration created successfully: {:?}", config);
///     }
///     Err(e) => {
///         eprintln!("Failed to create default configuration: {:?}", e);
///     }
/// }
/// ```
fn create_default(path: &PathBuf) -> Result<Config, ConfigError> {
    let config_file = File::create(path);
    match config_file {
        Ok(mut file) => {
            let pretty_config = PrettyConfig::new()
                .indentor("\t")
                .struct_names(false);

            let default_config = Config::default();
            let default_string = ron::ser::to_string_pretty(&default_config, pretty_config).map_err(|_| ConfigError::NoConfigOrFlag)?;
            file.write_all(default_string.as_bytes()).map_err(|_| ConfigError::NoConfigOrFlag)?;
            Ok(default_config)
        }
        Err(_) => Err(ConfigError::NoConfigOrFlag)
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempts to create a `Config` instance from a specified path or dynamically locate
    /// the configuration file by searching upwards within the directory structure.
    ///
    /// # Arguments
    /// * `config_path` - An optional path reference to a configuration file. If `None` is provided,
    ///    the function attempts to dynamically locate a configuration file based on the execution directory.
    ///
    /// # Returns
    /// * `Ok(Self)` - A successfully parsed and loaded `Config` object.
    /// * `Err(ConfigError)` - If any error occurs while locating, reading, or parsing the configuration file.
    ///
    /// # Behavior
    /// If the `config_path` is `None`:
    /// - The function identifies the directory containing the executing binary and
    ///   traverses upwards in the directory structure.
    /// - It searches for the presence of directories named `chat_shared`, `chat_server`,
    ///   and `chat_client` to locate a valid project structure.
    /// - When the project structure is found, it looks for the default configuration file
    ///   specified by `DEFAULT_CONFIG_FILE`.
    /// - If the configuration file exists and is readable, its contents are parsed into
    ///   a `Config` object using the `ron` format.
    /// - If the configuration file is missing or unreadable, it creates a default configuration
    ///   file in the expected location by invoking `create_default`.
    ///
    /// If the `config_path` is provided:
    /// - It attempts to open the file at the specified path.
    /// - Reads the file contents and parses it into a `Config` object using the `ron` format.
    ///
    /// # Errors
    /// * `ConfigError::NoConfigOrFlag` - If the configuration file path is invalid, missing, or
    ///   not located in a valid directory structure during dynamic discovery.
    /// * `ConfigError::ConfigReadFailed` - If the function fails to read the file contents.
    /// * `ConfigError::ConfigParseFailed` - If the function fails to parse the configuration file.
    ///
    /// # Notes
    /// - The function assumes a specific directory structure and configuration file naming convention.
    /// - In cases where dynamic discovery fails, ensure the `DEFAULT_CONFIG_FILE` exists and is accessible.
    /// - The `chat_shared`, `chat_server`, and `chat_client` directories are used as indicators to identify
    ///   the valid project structure.
    ///
    /// # Examples
    /// ```rust
    /// // Attempt to load configuration from a specified path
    /// let config = Config::from_path(Some(Path::new("path/to/config.ron")));
    ///
    /// // Attempt to locate and dynamically load configuration
    /// let config = Config::from_path(None);
    ///
    /// match config {
    ///     Ok(cfg) => println!("Configuration loaded successfully!"),
    ///     Err(err) => eprintln!("Failed to load configuration: {:?}", err),
    /// }
    /// ```
    pub fn from_path(config_path: Option<&Path>) -> Result<Self, ConfigError> {
        let mut config_file: File;
        if let None = config_path {
            let first_arg = std::env::args().nth(0).unwrap();
            let mut working_path = Path::new(&first_arg).parent().unwrap().canonicalize().unwrap();
            if !&working_path.is_dir() {
                println!("Not a directory!");
                return Err(ConfigError::NoConfigOrFlag);
            }

            loop {
                let dir_children = fs::read_dir(&working_path)
                    .map_err(|_| ConfigError::NoConfigOrFlag)?
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_dir())
                    .map(|entry| entry.file_name().to_string_lossy().to_string())
                    .collect::<Vec<String>>();

                if dir_children.contains(&"chat_shared".to_string())
                    && dir_children.contains(&"chat_server".to_string())
                    && dir_children.contains(&"chat_client".to_string()) {
                    let config_path = working_path.join(DEFAULT_CONFIG_FILE);


                    let config = File::open(&config_path)
                        .ok()
                        .and_then(|mut file| {
                            let mut contents = String::new();
                            file.read_to_string(&mut contents).ok()?;
                            ron::from_str::<Config>(&contents).ok()
                        })
                        .unwrap_or_else(|| {
                            create_default(&config_path).unwrap()
                        });

                    return Ok(config);
                }

                working_path = working_path.parent()
                    .ok_or(ConfigError::NoConfigOrFlag)?
                    .canonicalize().unwrap();
            }
        } else {
            config_file = match File::open(config_path.unwrap()) {
                Ok(file) => file,
                Err(_) => return Err(ConfigError::NoConfigOrFlag),
            }
        }

        let mut contents = String::new();

        config_file.read_to_string(&mut contents).map_err(|_| ConfigError::ConfigReadFailed)?;

        println!("{:?}", contents);

        ron::from_str::<Config>(&contents).map_err(|_| ConfigError::ConfigParseFailed)
    }

    /// Retrieves the IP address from the configuration, prioritizing IPv6 over IPv4.
    ///
    /// This function checks if an IPv6 address is available (`self.host_ipv6`).
    /// If present, it returns the IPv6 address as a `String`.
    /// If no IPv6 address is found, it then checks for the availability of an IPv4
    /// address (`self.host_ipv4`). If one is present, it returns the IPv4 address
    /// as a `String`.
    ///
    /// # Returns
    /// * `Ok(String)` - The IP address (IPv6 or IPv4) as a `String`.
    /// * `Err(ConfigError::MissingHostIp)` - If neither IPv6 nor IPv4 addresses are available.
    ///
    /// # Errors
    /// This function will return an error of type `ConfigError::MissingHostIp` if
    /// both `self.host_ipv6` and `self.host_ipv4` are `None`.
    ///
    /// # Example
    /// ```rust
    /// use some_crate::{YourStruct, ConfigError};
    ///
    /// let config = YourStruct {
    ///     host_ipv6: Some("::1".to_string()),
    ///     host_ipv4: Some("127.0.0.1".to_string()),
    /// };
    ///
    /// let ip = config.get_ip();
    /// match ip {
    ///     Ok(addr) => println!("IP Address: {}", addr),
    ///     Err(e) => eprintln!("Error: {:?}", e),
    /// }
    /// ```
    ///
    /// In this example:
    /// - If `host_ipv6` is `Some("::1")`, it will print: `IP Address: ::1`.
    /// - If `host_ipv6` is `None` and `host_ipv4` is `Some("127.0.0.1")`, it will print: `IP Address: 127.0.0.1`.
    /// - If both are `None`, it will return an error `Err(ConfigError::MissingHostIp)`.
    pub fn get_ip(&self) -> Result<String, ConfigError> {
        if let Some(ip) = self.host_ipv6 {
            return Ok(ip.to_string());
        }

        if let Some(ip) = self.host_ipv4 {
            return Ok(ip.to_string());
        }

        Err(ConfigError::MissingHostIp)
    }
}