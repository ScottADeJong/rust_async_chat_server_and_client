use chat_shared::{Config, User};
use std::sync::Arc;
use tokio::{
    io::ErrorKind,
    sync::Mutex,
    sync::mpsc::{Receiver, Sender},
};

// define a type to make this easier to work with
type Clients = Arc<Mutex<Vec<Arc<User>>>>;

// Get's a message from the buffer
pub fn get_message_from_buffer(buffer: &[u8]) -> Result<String, String> {
    match String::from_utf8(buffer.iter().filter(|n| **n != 0).copied().collect()) {
        Ok(s) => Ok(s),
        Err(e) => Err(e.to_string()),
    }
}

// Process a command string sent from the client
// Currently only returns OK, but error handling should be added
pub async fn process_command(command: &str, user: &Arc<User>) -> Result<(), String> {
    let args: Vec<&str> = command.split_whitespace().collect();
    if let Some(c) = args.first() {
        match *c {
            ":quit" => {
                let mut is_active = user.is_active.lock().await;
                *is_active = false;
            }
            ":name" => {
                let mut nickname = user.member.nick_name.lock().await;

                match args.len() {
                    n if n <= 1 => *nickname = None,
                    _ => *nickname = Some(args[1].to_string()),
                }
            }
            // Should message user that the command was not recognized
            _ => (),
        }
    }
    Ok(())
}

// Handle the writing to the attached clients
// Reads from the thread receiver and writes using the clients vec
pub async fn handle_writes(config: Arc<Config>, mut rx: Receiver<String>, clients: Clients) {
    // Exit if our receiver is closed
    while let Some(message) = rx.recv().await {
        let guard = clients.lock().await;
        for client in guard.iter() {
            let mut buff = message.clone().into_bytes();
            buff.resize(config.msg_size as usize, 0);

            if let Some(socket) = client.socket.as_ref()
                && socket.try_write(&buff).is_err()
            {
                continue;
            }
        }
    }
}

// Read messages from our client, parse them and where appropriate
// put send to the writer thread
pub async fn handle_client(
    config: Arc<Config>,
    user: Arc<User>,
    tx: Sender<String>,
    clients: Clients,
) {
    println!("Starting thread for {}", user.member.address);
    let mut buffer = vec![0; config.msg_size as usize];

    loop {
        {
            let is_active = user.is_active.lock().await;
            if !*is_active {
                break;
            }
        }

        let socket = user.socket.as_ref().unwrap();
        socket.readable().await.expect("Failed to check socket");
        let message = match user.socket.as_ref().unwrap().try_read(&mut buffer) {
            Ok(_) => get_message_from_buffer(&buffer),
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        };

        let message = match message {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        };

        // if the contents of msg match the command string, run process_command
        let message_result = match message.trim() {
            n if n.starts_with(":") => process_command(n, &user).await,
            "" => continue,
            _ => send_message(message, &user, &tx).await,
        };

        if let Err(e) = message_result {
            eprintln!("{e}");
            break;
        }
    }

    // if we get here, indicate we are closing the connection and remove
    // the client from the client's list
    println!("closing connection with: {}", user.member.address);
    remove_client(clients, user).await;
}

// function that removes the associated client from the client's list
pub async fn remove_client(clients: Clients, user: Arc<User>) {
    let mut remove_index: Option<usize> = None;
    let mut guard = clients.lock().await;
    for (index, client) in guard.iter().enumerate() {
        if Arc::ptr_eq(client, &user) {
            remove_index = Some(index);
            break;
        }
    }
    if let Some(index) = remove_index {
        guard.remove(index);
    }
}

// Sends messages on our sender to our writer thread
pub async fn send_message(
    message: String,
    user: &Arc<User>,
    tx: &Sender<String>,
) -> Result<(), String> {
    let message = format!("{}: {}", user.get_display_name().await, message);
    let message = message.replace('"', "");

    if tx.send(message).await.is_err() {
        eprintln!("closing connection with: {}", user.get_display_name().await);
        return Err(String::from("Failed to write message"));
    }
    Ok(())
}
