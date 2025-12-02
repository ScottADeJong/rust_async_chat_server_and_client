use chat_client::*;
use chat_shared::{Config, Message, User};
use std::{env::args, process, sync::Arc};
use tokio::{net::TcpStream, spawn, sync::mpsc};

#[tokio::main]
async fn main() {
    // Get the config file path from the command line arguments.
    // Then match the result and either load the config from the file the path supplied or request default.
    let config = match chat_shared::get_config_path(args()) {
        Some(config) => Config::from_path(Some(config.as_ref())),
        None => Config::from_path(None),
    };

    // If the config is not valid, print the error and exit.
    let config = match config {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };

    // If the config is valid, get the ip address and port from the config.
    // If the ip address is not valid, print the error and exit.
    let host_ip = config.get_ip().unwrap_or_else(|_| {
        eprintln!("Invalid IP address");
        process::exit(1);
    });

    // Create the address string to connect to.
    let address = format!("{}:{}", host_ip, config.host_port).replace('"', "");

    // Create a shared config and user object to pass to our threads
    let config = Arc::new(config);
    // Open our stream or die trying
    let client = TcpStream::connect(address)
        .await
        .expect("Stream failed to connect");

    // Create a shared user object to pass to our threads
    let user = Arc::new(User::from(client, None));

    // Open our thread communication channels
    let (tx, rx) = mpsc::channel::<Message>(32);

    // spawn off our routine that sends messages to the server
    spawn(send_to_server(Arc::clone(&config), rx, Arc::clone(&user)));
    // spawn off our routine that gets messages from the server
    spawn(get_message_from_server(
        Arc::clone(&config),
        Arc::clone(&user),
    ));

    println!("Welcome to chat!!!!");
    // Start our routine that gets a message from stdin and sends to the send_to_server thread
    read_and_send(tx.clone(), Arc::clone(&user)).await;
}
