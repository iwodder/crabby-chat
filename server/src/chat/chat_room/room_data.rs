use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use tungstenite::Message;
use crate::chat::chat_room::Extractor;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Clone)]
pub struct ChatData{
    room_id: Uuid,
    room_name: String,
    owner_id: String,
    users: Arc<Mutex<HashMap<String, Sender<Message>>>>,
    history: Arc<RwLock<Vec<Message>>>
}

impl ChatData {
    pub fn new(name: String, owner_id: String) -> Self {
        ChatData {
            room_id: Uuid::new_v4(),
            room_name: name,
            owner_id,
            users: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new()))
        }
    }

    pub fn id(&self) ->  String {
        self.room_id.to_string()
    }

    pub fn name(&self) -> String {
        self.room_name.clone()
    }

    pub fn add_message(&mut self, new_msg: Message) {
        self.history.write().unwrap().push(new_msg);
    }

    pub fn users(&self) -> Arc<Mutex<HashMap<String, Sender<Message>>>> {
        self.users.clone()
    }

    pub fn add_user(&mut self, user_name: String, tx: Sender<Message>) {
        self.users.lock().unwrap().insert(user_name, tx);
    }

    pub fn is_owner(&self, owner_id: &String) -> bool {
        self.owner_id.eq(owner_id)
    }

    pub fn extract_room_data<T: Extractor>(&self, extractor: &mut T) {
        extractor.pass_name(self.room_name.clone());
        let mut users = vec![];
        for key in self.users.lock().unwrap().keys() {
            users.push(key.clone());
        }
        extractor.handle_users(users.iter());
    }
}

impl PartialEq for ChatData {
    fn eq(&self, other: &Self) -> bool {
        return self.room_name.eq(&other.room_name)
    }

    fn ne(&self, other: &Self) -> bool {
        return !self.eq(other)
    }
}

impl Eq for ChatData {}

impl Hash for ChatData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.room_name.hash(state);
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) where
        Self: Sized, {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {

}