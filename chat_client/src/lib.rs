// TODO: Fix send_to_server
use chat_shared::{Config, Message, User, message::MessageKind};
use std::{io, sync::Arc, thread::sleep, time::Duration};
use tokio::{
    io::ErrorKind,
    sync::mpsc::{Receiver, Sender},
};

// Helper function to translate the buffer to utf8 and print to the console
pub async fn get_and_print_message(buffer: Vec<u8>, user: &Arc<User>) {
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
pub async fn read_and_send(tx: Sender<Message>, user: Arc<User>) {
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
        let message_kind: MessageKind;
        if buff.starts_with(':') {
            message_kind = MessageKind::Command;
            if buff.contains(":name ") {
                let mut nickname = user.nick_name.lock().await;
                *nickname = Some(buff.split_whitespace().nth(1).unwrap().to_string());
            }
        } else {
            message_kind = MessageKind::Message;
        }

        // Send to our receiver thread
        let message = Message::from_string(user.client.clone(), buff, message_kind);

        tx.send(message).await.expect("Couldn't send the message");
    }

    sleep(Duration::new(0, 100));
}

pub async fn get_message_from_server(config_handle: Arc<Config>, user: Arc<User>) {
    let socket = user.socket.as_ref().unwrap();
    while socket.readable().await.is_ok() {
        let mut buffer = vec![0; config_handle.msg_size as usize];
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
// TODO: Send the message struct instead of the content array
pub async fn send_to_server(config: Arc<Config>, mut rx: Receiver<Message>, user: Arc<User>) {
    while let Some(message) = rx.recv().await {
        if let Ok(buff) = ron::to_string(&message) {
            let mut buff = buff.into_bytes();
            buff.resize(config.msg_size as usize, 0);
            let socket = user.socket.as_ref().unwrap();

            socket.writable().await.expect("Could not check writable");
            socket.try_write(&buff).expect("writing to socket failed");
        }
    }
}
