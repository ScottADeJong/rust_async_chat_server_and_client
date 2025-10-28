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

fn process_command(command: &str, user: &mut User) {
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
}

async fn handle_writes(mut rx: Receiver<String>, clients: Arc<Mutex<Vec<Arc<TcpStream>>>>) {
    loop {
        if let Some(msg) = rx.recv().await {
            for client in clients.lock().await.iter() {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.try_write(&buff).expect("Failed to write");
            }
        }
    }
}

async fn handle_client(socket: Arc<TcpStream>, tx: mpsc::Sender<String>, addr: String) {
    let mut user = User::new(addr);
    let mut buff = vec![0; MSG_SIZE];

    loop {
        // Blocks until socket has something to read
        socket.readable().await.expect("Failed to check socket");

        // Assing the result of trying to get a utf8 string from the buffer to msg
        let msg = match socket.try_read(&mut buff) {
            Ok(_) => String::from_utf8(buff.iter().filter(|n| **n != 0).copied().collect()),
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(_) => {
                eprintln!("closing connection with: {}", user.nick_name);
                break;
            }
        };

        // Assign the utf8 value to msg or print the error and continue
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        // if the contents of msg match the command string run process_command
        match msg.trim() {
            n if n.starts_with(":") => process_command(n, &mut user),
            "" => continue,
            _ => {
                let msg = format!("{}: {}", user.nick_name, msg);
                let msg = msg.replace('"', "");
                tx.send(msg).await.expect("Failed to write message");
            }
        };

        if !user.is_active {
            println!("closing connection with: {}", user.nick_name);
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let server = TcpListener::bind(LOCAL)
        .await
        .expect("Listener failed to bind");

    let clients = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel::<String>(32);

    tokio::spawn(handle_writes(rx, clients.clone()));

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
