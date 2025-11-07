use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use ron::ser::PrettyConfig;
use crate::ConfigError;
use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG_FILE: &str = "env/config.ron";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub host_ipv4: Option<Ipv4Addr>,
    pub host_ipv6: Option<Ipv6Addr>,
    pub host_port: usize,
    pub msg_size: u8,
    pub prefix: char,
}

impl Default for Config {
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

        ron::from_str::<Config>(&contents).map_err(|_| ConfigError::ConfigParseFailed)
    }

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