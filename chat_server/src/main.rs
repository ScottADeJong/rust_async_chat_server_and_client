use std::sync::Arc;
use tokio::io::ErrorKind;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc, mpsc::Receiver};

// constants for message size and network address
const LOCAL: &str = "127.0.0.1:7070";
const MSG_SIZE: usize = 255;

// define a type to make this easier to work with
type Clients = Arc<Mutex<Vec<Arc<TcpStream>>>>;

// Struct to hold the user information
struct User {
    nick_name: String,
    is_active: bool,
}

impl User {
    fn new(nick_name: String) -> Self {
        Self {
            nick_name,
            is_active: true,
        }
    }

    fn disconnect(&mut self) {
        self.is_active = false;
    }
}

// Get's a message from the buffer
fn get_message_from_buffer(buffer: &[u8]) -> Result<String, String> {
    match String::from_utf8(buffer.iter().filter(|n| **n != 0).copied().collect()) {
        Ok(s) => Ok(s),
        Err(e) => Err(e.to_string()),
    }
}

// Process a command string that is sent from the client
// Currently only returns OK, but error handling should be added
fn process_command(command: &str, user: &mut User) -> Result<(), String> {
    let command: Vec<&str> = command.split_whitespace().collect();
    if let Some(c) = command.first() {
        match *c {
            ":quit" => user.disconnect(),
            ":name" => {
                user.nick_name = match command.get(1) {
                    Some(n) => n.to_string(),
                    None => String::from("unknown"),
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
async fn handle_writes(mut rx: Receiver<String>, clients: Clients) {
    // Exit if our receiver is closed
    println!("Starting handle_writes thread");
    while let Some(message) = rx.recv().await {
        let guard = clients.lock().await;
        for client in guard.iter() {
            let mut buff = message.clone().into_bytes();
            buff.resize(MSG_SIZE, 0);
            if client.try_write(&buff).is_err() {
                break;
            }
        }
    }
    println!("Ending handle_writes thread");
}

// Read messages from our client, parse them and where appropriate
// put send the to the writer thread
async fn handle_client(
    socket: Arc<TcpStream>,
    tx: mpsc::Sender<String>,
    addr: String,
    clients: Clients,
) {
    println!("Starting thread for {addr}");
    let mut user = User::new(addr.clone());
    let mut buffer = vec![0; MSG_SIZE];

    // Exit when our user is made inactive
    while user.is_active {
        // Blocks until socket has something to read
        socket.readable().await.expect("Failed to check socket");
        let message = match socket.try_read(&mut buffer) {
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

        // if the contents of msg match the command string run process_command
        let message_result = match message.trim() {
            n if n.starts_with(":") => process_command(n, &mut user),
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
    println!("closing connection with: {addr}");
    remove_client(clients, &socket).await;
}

// function that removes the associated client from the client's list
async fn remove_client(clients: Clients, socket: &Arc<TcpStream>) {
    let mut remove_index: Option<usize> = None;
    let mut guard = clients.lock().await;
    for (index, client) in guard.iter().enumerate() {
        if Arc::ptr_eq(client, socket) {
            remove_index = Some(index);
            break;
        }
    }
    if let Some(index) = remove_index {
        guard.remove(index);
    }
}

// Sends messages on our sender to our writer thread
async fn send_message(
    message: String,
    user: &User,
    tx: &mpsc::Sender<String>,
) -> Result<(), String> {
    let message = format!("{}: {}", user.nick_name, message);
    let message = message.replace('"', "");
    if tx.send(message).await.is_err() {
        eprintln!("closing connection with: {}", user.nick_name);
        return Err(String::from("Failed to write message"));
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    // Set up our listener or die trying
    let server = TcpListener::bind(LOCAL)
        .await
        .expect("Listener failed to bind");

    // Create our list of clients Needs to be Arc of Mutex of Arcs
    // so that the send trait is repected throughout
    let clients = Arc::new(Mutex::new(Vec::new()));
    // set up the sender and receiver for our threads
    let (tx, rx) = mpsc::channel::<String>(32);

    // spawn off our writer
    tokio::spawn(handle_writes(rx, Arc::clone(&clients)));

    // Loop until our listener fails
    while let Ok((socket, addr)) = server.accept().await {
        // log that a client connected
        let addr = addr.to_string();
        println!("Client {addr} connected");

        // put our socket in an Arc so it can be shared
        // and push it to the clients list
        let socket = Arc::new(socket);
        clients.lock().await.push(Arc::clone(&socket));

        // spawn off our client thread
        tokio::spawn(handle_client(
            socket,
            tx.clone(),
            addr,
            Arc::clone(&clients),
        ));
    }
}
