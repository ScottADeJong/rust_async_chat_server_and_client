use std::path::Path;

pub struct CliHandle {
    pub config: Option<Box<Path>>
}

fn get_config_file(mut args: std::env::Args) -> Option<Box<Path>> {
    let path_arg = args.nth(1).unwrap().clone().to_string();
    let config = Path::new(&path_arg);

    if !config.exists() {
        return None
    }

    Some(Box::from(config))
}

impl CliHandle {
    pub fn new(args: std::env::Args) -> Self {
        if args.len() < 2 {
            return Self {
                config: None
            }
        }

        Self {
            config: get_config_file(args)
        }
    }
}