use std::fs::File;

pub struct CliHandle {
    pub config: Option<File>
}

fn get_config_file(args: std::env::Args) -> Option<File> {
    let config = File::open(args.collect::<Vec<String>>()[1].clone());

    if config.is_err() {
        return None
    }

    Some(config.unwrap())
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