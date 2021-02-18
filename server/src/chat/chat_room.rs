use std::sync::{Arc, Mutex, mpsc};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use crate::user::User;
use tungstenite::{WebSocket, Message};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::spawn;
use std::io::Read;

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

    //Room implementation which uses a tcp listener
    pub fn run_room(&mut self, conn: TcpListener) {
        let (tx, rx) = mpsc::channel();
        self.run_receiver(rx);
        for stream in conn.incoming() {
            let mut buff = [0;1024];
            let mut tcpStream = stream.unwrap();
            tcpStream.read(&mut buff);
            println!("Buff was >> {}", String::from_utf8_lossy(&buff));
            self.join_room(tcpStream, tx.clone());
        }
    }

    pub fn run_room_chan(&mut self, new_client: Receiver<TcpStream>) {
        let (tx, rx) = mpsc::channel();
        self.run_receiver(rx);
        loop {
            if let Ok(client) = new_client.recv() {
                println!("Accepting new user into the room.");
                self.join_room(client, tx.clone());
            } else {
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
        ws.write_message(Message::text(String::from("Enter your name>>"))).unwrap();
        let name = ws.read_message().unwrap().into_text().unwrap();


        let (user_tx, user_rx) = mpsc::channel();
        let mut new_user = User::new(name);
        self.users.lock().unwrap().insert(new_user.name(), user_tx);
        thread::spawn(move || {
            new_user.run_user(ws, tx, user_rx);
        });
    }

}
