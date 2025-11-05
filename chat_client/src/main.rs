use chat_shared::handles::{CliHandle, ConfigHandle};
use chat_shared::objects::User;
use std::env::args;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use std::{io, process};
use tokio::io::ErrorKind;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::mpsc::{self, Receiver, Sender};

// Helper function to translate the buffer to utf8 and print to the console
async fn get_and_print_message(buffer: Vec<u8>, user: &Arc<User>) {
    // Translate buffer to a vec
    let message: Vec<u8> = buffer.into_iter().filter(|n| *n != 0).collect();
    // Translate the vec to a utf8 string
    let message = String::from_utf8(message).expect("Invalid utf8 message");
    // If the message is not empty and is not sent by us, print it
    let display_name = user.get_display_name().await;
    if !message.is_empty() && !message.starts_with(format!("{}: ", display_name).as_str()) {
        println!("-->{}", message);
    }
}

// This function handles getting information from
// stdin and sending it to the server
async fn read_and_send(tx: Sender<String>, user: Arc<User>) {
    // Create a buffer to control our loop and to collect
    // the message to send
    let mut buff = String::new();

    // Loop until we choose to quit
    while buff.trim() != ":quit" {
        buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("reading from stdin failed");

        let buff = buff.trim().to_string();
        if buff.contains(":name ") {
            let mut nickname = user.nickname.lock().await;
            *nickname = Some(buff.split_whitespace().nth(1).unwrap().to_string());
        }

        // Send to our receiver thread
        tx.send(buff).await.expect("Couldn't send the message");
    }

    sleep(Duration::new(0, 100));
}

async fn get_message_from_server(config_handle: Arc<ConfigHandle>, user: Arc<User>) {
    let socket = user.socket.as_ref().unwrap();
    while socket.readable().await.is_ok() {
        let mut buffer = vec![0; config_handle.get_value_usize("msg_size").unwrap()];
        match socket.try_read(&mut buffer) {
            Ok(0) => continue,
            Ok(_) => get_and_print_message(buffer, &user).await,
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(_) => {
                eprintln!("Connection with the server was severed");
                break;
            }
        }
    }
}

// check the receiver and if we have data, try to write it to the
// stream
async fn send_to_server(
    config_handle: Arc<ConfigHandle>,
    mut rx: Receiver<String>,
    user: Arc<User>,
) {
    while let Some(message) = rx.recv().await {
        let mut buff = message.into_bytes();
        buff.resize(config_handle.get_value_usize("msg_size").unwrap(), 0);
        let socket = user.socket.as_ref().unwrap();

        socket.writable().await.expect("Could not check writable");
        socket.try_write(&buff).expect("writing to socket failed");
    }
}

#[tokio::main]
async fn main() {
    let cli_handle = CliHandle::new(args());
    let config_handle = match cli_handle.config {
        Some(config) => ConfigHandle::new(Some(config)),
        None => ConfigHandle::new(None),
    };

    let config_handle = match config_handle {
        Ok(config_handle) => Arc::new(config_handle),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let address = format!(
        "{}:{}",
        config_handle.get_value_string("host_ip").unwrap(),
        config_handle.get_value_string("host_port").unwrap()
    )
    .replace('"', "");

    // Open our stream or die trying
    let client = TcpStream::connect(address)
        .await
        .expect("Stream failed to connect");

    let user = Arc::new(User::from(client, None));

    // Open our thread communication channels
    let (tx, rx) = mpsc::channel::<String>(32);

    // spawn off our routine that sends messages to the server
    spawn(send_to_server(
        Arc::clone(&config_handle),
        rx,
        Arc::clone(&user),
    ));
    // spawn off our routine that gets messages from the server
    spawn(get_message_from_server(
        Arc::clone(&config_handle),
        Arc::clone(&user),
    ));

    println!("Welcome to chat!!!!");
    // Start our routine that gets a message from stdin and sends to the send_to_server thread
    read_and_send(tx.clone(), Arc::clone(&user)).await;
}
