use std::io;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::io::ErrorKind;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, Receiver, Sender};

// Define a constant for our connection address
// should maybe go in a config file
const LOCAL: &str = "127.0.0.1:7070";
const MSG_SIZE: usize = 255;

// Get the user nickname from the command line
fn get_name_from_args() -> Result<String, String> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1) {
        Some(s) => Ok(s.to_string()),
        None => {
            let error = format!("usage: {} user_name", args[0]);
            Err(error)
        }
    }
}
// Helper function to translate the buffer to utf8 and print to the console
fn get_and_print_message(buffer: Vec<u8>, user: &String) {
    // Translate buffer to a vec
    let message: Vec<u8> = buffer.into_iter().filter(|n| *n != 0).collect();
    // Translate the vec to a utf8 string
    let message = String::from_utf8(message).expect("Invalid utf8 message");
    // If the message is not empty and is not sent by us, print it
    if !message.is_empty() && !message.contains(format!("{}: ", user).as_str()) {
        println!("--->{}", message);
    }
}

// This function handles getting information from
// stdin and sending it to the server
async fn read_and_send(tx: Sender<String>, user: Arc<Mutex<String>>) {
    // Set our nickname when we start up
    {
        let user = user.lock().await;
        tx.send(format!(":name {}", *user))
            .await
            .expect("Couldn't set nickname");
    }
    // Create a bufffer to control our loop and to collect
    // the message to send
    let mut buff = String::new();

    // Loop until we choos to quit
    while buff.trim() != ":quit" {
        buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("reading from stdin failed");

        let mut buff = buff.trim().to_string();
        if buff.contains(":name ") {
            let mut user = user.lock().await;
            *user = buff
                .split_whitespace()
                .map(|x| x.to_string())
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            buff = format!(":name {user}");
        }

        // send to our receiver thread
        tx.send(buff).await.expect("Couldn't send the message");
    }
}

async fn get_message_from_server(client: Arc<TcpStream>, user: Arc<Mutex<String>>) {
    // Loop until the client is no longer readable/connected
    while client.readable().await.is_ok() {
        let mut buffer = vec![0; MSG_SIZE];
        match client.try_read(&mut buffer) {
            // Skip zero length buffer
            Ok(0) => continue,
            Ok(_) => get_and_print_message(buffer, &user.lock().await.to_string()),
            // This error is ignored because it's really just a real-time
            // warning saying that nothing is available at the moement, so
            // skipping this polling attempt.
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(_) => {
                eprintln!("connection with server was severed");
                break;
            }
        }
    }
}

// check the receiver and if we have data, try to write it to the
// stream
async fn send_to_server(mut rx: Receiver<String>, client: Arc<TcpStream>) {
    while let Some(message) = rx.recv().await {
        let mut buff = message.into_bytes();
        buff.resize(MSG_SIZE, 0);
        client.writable().await.expect("Could not check writable");
        client.try_write(&buff).expect("writing to socket failed");
    }
}

#[tokio::main]
async fn main() {
    let user = get_name_from_args().expect("{e}");

    // Open our stream or die trying
    let client = TcpStream::connect(LOCAL)
        .await
        .expect("Stream failed to connect");
    // Put our stream in a shareable smart pointer
    let client = Arc::new(client);
    let user = Arc::new(Mutex::new(user));
    //
    // Open our thread communication channels
    let (tx, rx) = mpsc::channel::<String>(32);

    // spawn off our routine that sends messags to the server
    spawn(send_to_server(rx, Arc::clone(&client)));
    // spawn off our routine that gets messages from the server
    spawn(get_message_from_server(
        Arc::clone(&client),
        Arc::clone(&user),
    ));

    println!("Welcome to chat!!!!");
    // Start our routine that gets a message from stdin and sends to the send_to_server thread
    read_and_send(tx.clone(), Arc::clone(&user)).await;
    sleep(Duration::new(0, 100));
}
