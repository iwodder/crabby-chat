use std::sync::{Arc, Mutex, mpsc};
use std::net::TcpStream;
use std::collections::HashMap;
use crate::user::User;
use tungstenite::Message;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub struct ChatRoom {
    name: String,
    users: Arc<Mutex<HashMap<String, Sender<Message>>>>
}

impl ChatRoom {
    pub fn new(name: String) -> Self {
        ChatRoom {
            name,
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run_room(&mut self, new_client: Receiver<TcpStream>) {
        let (tx, rx) = mpsc::channel();
        self.run_receiver(rx);
        loop {
            if let Ok(client) = new_client.recv() {
                println!("Accepting new user into the room.");
                self.join_room(client, tx.clone());
            } else {
                //receive error means the sending end closed
                break;
            }
        }
    }

    //Relays messages to all clients in the room
    fn run_receiver(&mut self, rx: Receiver<Message>) {
        let clone = self.users.clone();
        thread::Builder::new().name(String::from("Receiver"))
            .spawn(move || {
                println!("Running receiver thread");
               loop {
                   let mut msg = rx.recv().unwrap();
                   println!("Message received");
                   for user in clone.lock().unwrap().iter() {
                       user.1.send(msg.clone());
                   }
                   println!("Message sent");
               }
            });
    }

    fn join_room(&mut self, stream: TcpStream, tx: Sender<Message>) {
        let mut ws = tungstenite::accept(stream).unwrap();
        ws.write_message(Message::text(String::from("Enter your name >>"))).unwrap();
        let name = ws.read_message().unwrap().into_text().unwrap();

        let (user_tx, user_rx) = mpsc::channel();
        let mut new_user = User::new(name);
        self.users.lock().unwrap().insert(new_user.name(), user_tx);
        thread::spawn(move || {
            new_user.run_user(ws, tx, user_rx);
        });
    }

}
