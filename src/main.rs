use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 255;
const SLEEP: u64 = 100;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(SLEEP));
}

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
            _ => (),
        }
    }
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone client"));

            thread::spawn(move || {
                let mut user = User::new(addr.to_string());
                loop {
                    let mut buff = vec![0; MSG_SIZE];
                    if !user.is_active {
                        println!("got here");
                        socket.shutdown(std::net::Shutdown::Both).unwrap();
                    }

                    match socket.read(&mut buff) {
                        Ok(_) => {
                            let msg =
                                String::from_utf8(buff.into_iter().filter(|n| *n != 0).collect())
                                    .expect("Invalid utf8 message");
                            match msg.trim() {
                                "" => (),
                                n if n.starts_with(":") => process_command(n, &mut user),
                                _ => {
                                    let msg = format!("{}: {}", user.nick_name, msg);
                                    let msg = msg.replace('"', "");
                                    tx.send(msg).expect("failed to send msg to rx");
                                }
                            }
                        }
                        // This error is ignored because it's really just a real-time
                        // warning saying that nothing is available at the moement, so
                        // skipping this polling attempt.
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!("closing connection with: {}", addr);
                            break;
                        }
                    }

                    sleep();
                }
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }

        sleep();
    }
}
