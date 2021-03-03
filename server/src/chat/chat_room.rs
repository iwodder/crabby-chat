pub mod room_data;

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
use crate::chat::chat_room::room_data::ChatData;

pub struct ChatRoom {
   data: ChatData,
   tx: Option<Sender<Message>>
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
                info!("Sending end closed, terminating room");
                self.tx.as_ref().unwrap().send(Message::Close(None));
                break;
            }
        }
    }

    fn run_receiver(mut room_data: ChatData, rx: Receiver<Message>) {
        thread::Builder::new()
            .name(format!("{}-Receiver", room_data.name()))
            .spawn(move || {
                info!("Running receiver thread");
                loop {
                    if let Ok(msg) = rx.recv() {
                        room_data.add_message(msg.clone());
                        info!("Message received");
                        match msg {
                            Message::Text(txt) => {
                                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&txt) {
                                    ChatRoom::send_msg_to_users(
                                        room_data.users(), Some(&chat_msg.from),
                                        Message::text(txt));
                                }
                                info!("Message sent");
                            },
                            Message::Close(frame) => {
                                ChatRoom::send_msg_to_users(
                                    room_data.users(), None, Message::Close(frame));
                                info!("Close frame sent to users, receiver shutting down.");
                                break;
                            },
                            Message::Binary(bin) => (),
                            _ => ()
                        }
                    }
                }
            });
    }

    fn join_room(&mut self, stream: TcpStream, tx: Sender<Message>) {
        if let Ok(mut ws) = tungstenite::accept(stream) {
            ws.write_message(Message::text(String::from("Enter user info"))).unwrap();
            let data = ws.read_message().unwrap().into_text().unwrap();
            let json: ChatUser = serde_json::from_str(&data).unwrap();

            let mut new_user = User::new(json.name);
            self.new_user_joined_msg(new_user.name());

            let (user_tx, user_rx) = mpsc::channel();
            self.data.add_user(new_user.name(), user_tx);

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
        ChatRoom::send_msg_to_users(self.data.users(), None, msg)
    }

    fn send_msg_to_users(users: Arc<Mutex<HashMap<String, Sender<Message>>>>, excluded_user: Option<&str>, msg: Message) {
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
}

pub trait Extractor {
    fn pass_name(&mut self, name: String);
    fn handle_users(&mut self, users: std::slice::Iter<String>);
}

#[cfg(test)]
mod test {
    use std::sync::mpsc;
    use crate::chat::chat_room::ChatRoom;
    use crate::chat::chat_room::room_data::ChatData;
    use std::thread::{sleep, spawn};
    use std::time::Duration;

    #[test]
    fn closing_sender_closes_room() {
        let (tx, rx) = mpsc::channel();
        let mut room = ChatRoom::new(
            ChatData::new(String::from("room"), String::from("owner")));

        spawn(move || {
            sleep(Duration::from_millis(20_000));
            drop(tx);
        });

        // Since run_room blocks execution until clean up, if this test
        // doesn't run infinitely the thread closed and we are good.
        room.run_room(rx);
    }
}