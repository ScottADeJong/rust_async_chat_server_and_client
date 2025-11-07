pub mod handles;
pub mod errors;
pub mod objects;

pub use errors::*;
pub use objects::*;

/// Retrieves the configuration file path from the provided command-line arguments.
///
/// # Parameters
/// - `args`: An iterator over the command-line arguments (`std::env::Args`),
///   typically passed as `std::env::args()`, where the first argument is the program name.
///
/// # Returns
/// - `Some(Box<Path>)`: A boxed `Path` pointing to the configuration file if the conditions are met:
///   - At least one argument (aside from the program name) is provided.
///   - The path provided as the second argument exists.
/// - `None`: If either no configuration path is provided or the specified path does not exist.
///
/// # Behavior
/// - The function checks the `args` iterator:
///   - If fewer than two arguments are provided, it returns `None`.
/// - It extracts the second argument (expected to be the config file path) and attempts to create a `Path`.
/// - If the path does not exist, the function returns `None`. Otherwise, it wraps the path in a `Box` and returns it.
///
/// # Example
/// ```
/// fn main() {
///     let args = std::env::args();
///     match get_config_path(args) {
///         Some(config_path) => println!("Config file path: {}", config_path.display()),
///         None => println!("Invalid or missing config file path."),
///     }
/// }
/// ```
///
/// # Notes
/// - This function consumes the `args` iterator; later calls to access it will fail.
/// - Be cautious about file existence checks in environments with dynamic file system states.
///
/// # Platform-specific behavior
/// - The behavior of `Path::exists()` depends on the underlying operating system and its file system implementation.
pub fn get_config_path(mut args: std::env::Args) -> Option<Box<std::path::Path>> {
    if args.len() < 2 {
        return None;
    }

    let path_arg = args.nth(1).unwrap().clone().to_string();
    let config = std::path::Path::new(&path_arg);

    if !config.exists() {
         return None;
    }

    Some(Box::from(config))
}