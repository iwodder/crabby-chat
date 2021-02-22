use std::sync::{Arc, Mutex, mpsc, RwLock};
use std::net::TcpStream;
use std::collections::HashMap;
use crate::chat::chat_user::User;
use crate::chat::chat_data::{ChatUser, ChatMessage};
use tungstenite::Message;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use log::info;
use std::hash::{Hash, Hasher};

pub struct ChatRoom {
   data: ChatData,
   tx: Option<Sender<Message>>
}

#[derive(Clone)]
pub struct ChatData {
    pub name: String,
    users: Arc<Mutex<HashMap<String, Sender<Message>>>>,
    history: Arc<RwLock<Vec<Message>>>
}

impl PartialEq for ChatData {
    fn eq(&self, other: &Self) -> bool {
        return self.name.eq(&other.name)
    }

    fn ne(&self, other: &Self) -> bool {
        return !self.eq(other)
    }
}

impl Eq for ChatData {

}

impl Hash for ChatData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) where
        Self: Sized, {
        unimplemented!()
    }
}

impl ChatData {
    pub fn new(name: String) -> Self {
        ChatData {
            name,
            users: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new()))
        }
    }

    pub fn extract_room_data<T: Extractor>(&self, extractor: &mut T) {
        extractor.pass_name(self.name.clone());
        let mut users = vec![];
        for key in self.users.lock().unwrap().keys() {
            users.push(key.clone());
        }
        extractor.handle_users(users.iter());
    }
}

impl ChatRoom {
    pub fn new(data: ChatData) -> Self {
        ChatRoom {
            data,
            tx: None
        }
    }

    pub fn run_room(&mut self, new_client: Receiver<TcpStream>) {
        let (tx, rx) = mpsc::channel();
        ChatRoom::run_receiver(self.data.clone(), rx);
        self.tx = Some(tx.clone());
        self.process_new_clients(new_client, tx);
    }

    fn process_new_clients(&mut self, new_client: Receiver<TcpStream>, room_tx: Sender<Message>) {
        loop {
            if let Ok(client) = new_client.recv() {
                info!("Accepting new user into the room.");
                self.join_room(client, room_tx.clone());
            } else {
                //receive error means the sending end closed
                break;
            }
        }
    }

    fn run_receiver(room_data: ChatData, rx: Receiver<Message>) {
        thread::Builder::new()
            .name(format!("{}-Receiver", &room_data.name))
            .spawn(move || {
                info!("Running receiver thread");
                loop {
                    if let Ok(msg) = rx.recv() {
                        room_data.history.write().unwrap().push(msg.clone());
                        info!("Message received");
                        match msg {
                            Message::Text(txt) => {
                                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&txt) {
                                    ChatRoom::send_msg_to_users(
                                        &room_data.users, Some(&chat_msg.from),
                                        Message::text(txt));
                                }
                            },
                            Message::Close(frame) => (),
                            Message::Binary(bin) => (),
                            _ => ()
                        }
                    }
                   info!("Message sent");
                }
            });
    }

    fn send_msg_to_users(users: &Mutex<HashMap<String, Sender<Message>>>, excluded_user: Option<&str>, msg: Message) {
        let name = match excluded_user {
            Some(s) => s,
            None => ""
        };
        for (user_name, tx) in users.lock().unwrap().iter() {
            if !user_name.eq(name) {
                tx.send(msg.clone());
            }
        }
    }

    fn join_room(&mut self, stream: TcpStream, tx: Sender<Message>) {
        if let Ok(mut ws) = tungstenite::accept(stream) {
            ws.write_message(Message::text(String::from("Enter user info"))).unwrap();
            let data = ws.read_message().unwrap().into_text().unwrap();
            let json: ChatUser = serde_json::from_str(&data).unwrap();

            let mut new_user = User::new(json.name);
            self.new_user_joined_msg(new_user.name());

            let (user_tx, user_rx) = mpsc::channel();
            self.data.users.lock().unwrap().insert(new_user.name(), user_tx);

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
        ChatRoom::send_msg_to_users(&self.data.users, None, msg)
    }
}

pub trait Extractor {
    fn pass_name(&mut self, name: String);
    fn handle_users(&mut self, users: std::slice::Iter<String>);
}
