use std::path::Path;

pub mod handles;
pub mod errors;
pub mod objects;

pub use errors::*;
pub use objects::*;

// Export generic functions.
pub fn get_config_path(mut args: std::env::Args) -> Option<Box<Path>> {
    if args.len() < 2 {
        return None;
    }

    let path_arg = args.nth(1).unwrap().clone().to_string();
    let config = Path::new(&path_arg);

    if !config.exists() {
         return None;
    }

    Some(Box::from(config))
}