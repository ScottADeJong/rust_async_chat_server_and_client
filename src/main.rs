use std::sync::Arc;
use tokio::io::ErrorKind;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc, mpsc::Receiver};

const LOCAL: &str = "127.0.0.1:7070";
const MSG_SIZE: usize = 255;

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

async fn handle_writes(mut rx: Receiver<String>, clients: Arc<Mutex<Vec<Arc<TcpStream>>>>) {
    loop {
        let mut clients_to_remove = Vec::new();
        if let Some(message) = rx.recv().await {
            for (index, client) in clients.lock().await.iter().enumerate() {
                let mut buff = message.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                if client.try_write(&buff).is_err() {
                    eprintln!("Failed to write");
                    clients_to_remove.push(index);
                }
            }
        }
        let mut guard = clients.lock().await;
        for index in clients_to_remove {
            guard.remove(index);
        }
    }
}

async fn handle_client(
    socket: Arc<TcpStream>,
    tx: mpsc::Sender<String>,
    addr: String,
) -> Result<(), String> {
    let mut user = User::new(addr);
    let mut buffer = vec![0; MSG_SIZE];

    while user.is_active {
        // Blocks until socket has something to read
        socket.readable().await.expect("Failed to check socket");
        let message = match socket.try_read(&mut buffer) {
            Ok(_) => get_message_from_buffer(&buffer)?,
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(_) => {
                let error = format!("closing connection with: {}", user.nick_name);
                eprintln!("{error}");
                return Err(error);
            }
        };

        // if the contents of msg match the command string run process_command
        match message.trim() {
            n if n.starts_with(":") => process_command(n, &mut user)?,
            "" => continue,
            _ => send_message(message, &user, &tx).await?,
        };
    }
    println!("closing connection with: {}", user.nick_name);
    Ok(())
}

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

fn get_message_from_buffer(buffer: &[u8]) -> Result<String, String> {
    match String::from_utf8(buffer.iter().filter(|n| **n != 0).copied().collect()) {
        Ok(s) => Ok(s),
        Err(e) => Err(e.to_string()),
    }
}

#[tokio::main]
async fn main() {
    let server = TcpListener::bind(LOCAL)
        .await
        .expect("Listener failed to bind");

    let clients = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel::<String>(32);

    tokio::spawn(handle_writes(rx, Arc::clone(&clients)));

    loop {
        if let Ok((socket, addr)) = server.accept().await {
            let addr = addr.to_string();
            println!("Client {addr} connected");

            let socket = Arc::new(socket);
            clients.lock().await.push(Arc::clone(&socket));
            tokio::spawn(handle_client(socket, tx.clone(), addr));
        }
    }
}
