use std::sync::{Arc, Mutex, mpsc, RwLock};
use std::net::TcpStream;
use std::collections::HashMap;
use crate::user::User;
use crate::chat::chat_data::{ChatUser, ChatMessage, ChatRooms};
use tungstenite::Message;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub struct ChatRoom {
    name: String,
    users: Arc<Mutex<HashMap<String, Sender<Message>>>>
    history: Arc<RwLock<Vec<Message>>>
}

impl ChatRoom {
    pub fn new(name: String) -> Self {
        ChatRoom {
            name,
            users: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(RwLock::new(vec![]))
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
                let users = clone;
                println!("Running receiver thread");
                loop {
                    if let Ok(msg) = rx.recv() {
                        println!("Message received");
                        match msg {
                            Message::Text(txt) => {
                                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&txt) {
                                    ChatRoom::send_msg_to_users(
                                        &users, &chat_msg.from, Message::text(txt));
                                }
                            },
                            Message::Close(frame) => (),
                            Message::Binary(bin) => (),
                            _ => ()
                        }
                    }
                   println!("Message sent");
                }
            });
    }

    fn send_msg_to_users(users: &Arc<Mutex<HashMap<String, Sender<Message>>>>, excluded_user: &str, msg: Message) {
        for (user, tx) in users.lock().unwrap().iter() {
            if !user.eq(&excluded_user) {
               tx.send(msg.clone());
            }
        }
    }

    fn join_room(&mut self, stream: TcpStream, tx: Sender<Message>) {
        if let Ok(mut ws) = tungstenite::accept(stream) {
            ws.write_message(Message::text(String::from("Enter user info"))).unwrap();
            let data = ws.read_message().unwrap().into_text().unwrap();
            let json: ChatUser = serde_json::from_str(&data).unwrap();

            let (user_tx, user_rx) = mpsc::channel();
            let mut new_user = User::new(json.name);
            self.new_user_joined_msg(new_user.name());
            self.users.lock().unwrap().insert(new_user.name(), user_tx);
            thread::spawn(move || {
                new_user.run_user(ws, tx, user_rx);
            });
        }
    }

    fn new_user_joined_msg(&self, name: String) {
        let msg = ChatMessage {
            from: String::from("Admin"),
            msg: format!("New user, {}, joined the chat!", name)
        };
        let json = serde_json::to_string(&msg).unwrap();
        let msg= Message::text(json);
        self.msg_all_users(msg);
    }

    fn msg_all_users(&self, msg: Message) {
        for u in self.users.lock().unwrap().iter() {
            u.1.send(msg.clone());
        }
    }
}
