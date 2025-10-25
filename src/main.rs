use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:7070";
const MSG_SIZE: usize = 255;
const SLEEP: u64 = 100;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let user = match args.get(1) {
        Some(s) => s,
        None => {
            eprintln!("usage: {} user_name", args[0]);
            return;
        }
    };

    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    let thread_user = user.clone();
    thread::spawn(move || {
        loop {
            let mut buff = vec![0; MSG_SIZE];
            match client.read(&mut buff) {
                Ok(_) => {
                    let msg = String::from_utf8(buff.into_iter().filter(|n| *n != 0).collect())
                        .expect("Invalid utf8 message");
                    if !msg.is_empty() && !msg.contains(format!("{}: ", &thread_user).as_str()) {
                        println!("--->{:?}", msg);
                    }
                }
                // This error is ignored because it's really just a real-time
                // warning saying that nothing is available at the moement, so
                // skipping this polling attempt.
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("connection with server was severed");
                    break;
                }
            }

            match rx.try_recv() {
                Ok(msg) => {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    client.write_all(&buff).expect("writing to socket failed");
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }

            thread::sleep(Duration::from_millis(SLEEP));
        }
    });

    println!("Write a Message:");
    if tx.send(format!(":name {}", user)).is_err() {
        return;
    };

    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("reading from stdin failed");
        let msg = buff.trim().to_string();

        match tx.send(msg.clone()) {
            Ok(_) => {
                if msg == ":quit" {
                    thread::sleep(Duration::from_millis(500));
                    break;
                }
            }
            Err(_) => break,
        }
    }
    println!("Goodbye!");
}
