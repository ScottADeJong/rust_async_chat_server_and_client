use chat_server::*;
use chat_shared::Config;
use chat_shared::objects::User;
use std::env::args;
use std::process;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() {
    // Get the config file path from the command line arguments
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

    // Create a shared config object to pass to our threads
    let config = Arc::new(config);
    // Set up our listener or die trying
    let server = TcpListener::bind(&address)
        .await
        .expect("Listener failed to bind");

    println!("Server is listening on {}!", address);

    // Create our list of clients Needs to be Arc of Mutex of Arcs
    // so that the sent trait is respected throughout
    let clients = Arc::new(Mutex::new(Vec::new()));
    // set up the sender and receiver for our threads
    let (tx, rx) = channel::<String>(32);

    // spawn off our writer
    tokio::spawn(handle_writes(Arc::clone(&config), rx, Arc::clone(&clients)));

    // Loop until our listener fails
    while let Ok((socket, addr)) = server.accept().await {
        // log that a client connected
        let addr = addr.to_string();
        println!("Client {addr} connected");

        // put our socket in an Arc so it can be shared
        // and push it to the client's list
        let user = Arc::new(User::from(socket, Some(addr)));
        clients.lock().await.push(Arc::clone(&user));

        // spawn off our client thread
        tokio::spawn(handle_client(
            Arc::clone(&config),
            Arc::clone(&user),
            tx.clone(),
            Arc::clone(&clients),
        ));
    }
}
