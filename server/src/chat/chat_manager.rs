use crate::chat::chat_room::{ChatRoom, Extractor};
use crate::chat::chat_room::room_data::ChatData;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::net::{TcpListener, SocketAddr, TcpStream};
use threadpool::ThreadPool;
use std::sync::mpsc::{Sender, Receiver};
use std::io::{Write};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::borrow::Cow;
use log::info;
use crate::chat::chat_manager::Error::{TooManyRooms};
use crate::chat::name_extractor;
use std::fmt;
use crate::chat::chat_user::User;
use crate::chat::chat_data::RoomCreated;

const ROOM_LIMIT: usize = 10;
const ROOM_LIMIT_AND_MGR: usize = ROOM_LIMIT+1;

pub struct ChatManager {
    rooms: Arc<Mutex<HashMap<ChatData, Sender<TcpStream>>>>,
    started: AtomicBool,
    thread: Mutex<ThreadPool>
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            thread: Mutex::new(ThreadPool::new(ROOM_LIMIT_AND_MGR)),
            started: AtomicBool::new(false),
        }
    }

    pub fn run(&mut self, server_addr: SocketAddr) {
        if self.started.load(Ordering::Relaxed) {
            panic!("Illegal operation to start manager twice")
        } else {
            let rooms_clone = self.rooms.clone();
            self.thread.lock().unwrap().execute(move || {
                let conn = TcpListener::bind(server_addr).unwrap();
                info!("Chat server is up and running... waiting for connections.");
                for new_stream in conn.incoming() {
                    let mut stream = new_stream.unwrap();
                    if let Some(name) = name_extractor::get_room_name(&mut stream) {
                        for (room, tx) in rooms_clone.lock().unwrap().iter() {
                            if room.name().eq(&name) {
                                return tx.send(stream).unwrap();
                            }
                        }
                        stream.write(b"HTTP/1.1 404 NOT FOUND");
                    }
                }
            });
            self.started.store(true, Ordering::Relaxed);
        }
    }

    pub fn create_new_room(&mut self, name: String, owner_id: String) -> Result<RoomCreated, Error> {
        if self.too_many_rooms() {
           Err(Error::TooManyRooms)
        } else {
            self.create_room(name, owner_id)
        }
    }

    pub fn delete_room(&mut self, room_id: String, owner_id: String) -> Result<(), Error> {
        if self.is_valid_room_id(&room_id) {
            self.try_to_delete_room(room_id, owner_id)
        } else {
            Err(Error::RoomNotFound)
        }
    }

    pub fn name_is_available(&self, name: &String) -> bool {
        !self.name_is_unavailable(name)
    }

    pub fn list_rooms(&self) -> Vec<String> {
        let mut vec = vec![];
        for room in self.rooms.lock().unwrap().keys() {
            vec.push(room.name());
        }
        vec
    }

    pub fn get_room_data<T: Extractor>(&self, extractor: &mut T) {
        for (room, _) in self.rooms.lock().unwrap().iter() {
            room.extract_room_data(extractor);
        }
    }

    fn create_room(&mut self, name: String, owner_id: String) -> Result<RoomCreated, Error> {
        if self.name_is_unavailable(&name) {
            Err(Error::NameTaken)
        } else {

            let (room, client_rx) = mpsc::channel();
            let data = ChatData::new(name, owner_id);
            let result = RoomCreated {
                path: String::from(""),
                name: data.name(),
                id: data.id()
            };

            self.add_and_start_room(data, room, client_rx);
            Ok(result)
        }
    }

    fn add_and_start_room(&mut self, room_data: ChatData, room_tx: Sender<TcpStream>, room_rx: Receiver<TcpStream>) {
        self.add_room_to_map(room_data.clone(), room_tx);
        self.start_room_thread(room_data, room_rx);
    }

    fn add_room_to_map(&mut self, room_data: ChatData, room_tx: Sender<TcpStream>) {
        self.rooms.lock().unwrap().insert(room_data, room_tx);
    }

    fn start_room_thread(&mut self, room_data: ChatData, client_rx: Receiver<TcpStream>) {
        self.thread.lock().unwrap().execute(move || {
            let mut new_room = ChatRoom::new(room_data);
            new_room.run_room(client_rx);
        });
    }

    fn name_is_unavailable(&self, name: &String) -> bool {
        let name_filter = |data: &ChatData| {
          data.name().to_lowercase() == name.to_lowercase()
        };
        self.room_exists_based_on_predicate(name_filter)
    }

    fn is_valid_room_id(&self, room_id: &String) -> bool {
        self.room_exists_based_on_predicate(ChatManager::id_filter(room_id))
    }

    fn try_to_delete_room(&mut self, room_id: String, owner_id: String) -> Result<(), Error> {
        let key = self.rooms.lock().unwrap().keys().find(|d|{
            d.id().eq(&room_id)
        }).unwrap().clone();
        if key.is_owner(&owner_id) {
            let sender = self.rooms.lock().unwrap().remove(&key).unwrap();
            drop(sender);
            Ok(())
        } else {
            Err(Error::NotOwner)
        }
    }

    fn id_filter(room_id: &String) -> impl FnMut(&ChatData) -> bool {
        let clone = room_id.clone();
        move |data: &ChatData| {
            data.id().eq(&clone)
        }
    }

    fn room_exists_based_on_predicate<F>(&self, filter: F) -> bool where F: FnMut(&ChatData) -> bool,  {
        self.rooms.lock().unwrap().keys().any(filter)
    }

    fn too_many_rooms(&mut self) -> bool {
        self.rooms.lock().unwrap().len() >= ROOM_LIMIT
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    TooManyRooms,
    RoomNotFound,
    NameTaken,
    NotOwner
}

impl std::error::Error for Error{}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Error::TooManyRooms => write!(f, "Too many rooms running."),
            Error::RoomNotFound => write!(f, "Room doesn't exist."),
            Error::NameTaken => write!(f, "Name is already in use."),
            Error::NotOwner => write!(f, "Not authorized to delete room.")
        }
    }
}

#[cfg(test)]
mod test {
    use crate::chat::chat_manager::ChatManager;
    use crate::chat::chat_manager::Error;
    use std::net::{SocketAddr, IpAddr};

    #[test]
    fn can_create_up_to_ten_chat_rooms() {
        let owner_id = String::from("user-a");
        let mut cm = ChatManager::new();
        for idx in 0..10 {
            cm.create_new_room(format!("Room #{}", idx), owner_id.clone());
        }

        assert_eq!(10, cm.list_rooms().len())
    }

    #[test]
    fn creating_more_than_ten_rooms_causes_error() {
        let owner_id = String::from("user-a");
        let mut cm = ChatManager::new();
        for idx in 0..10 {
            cm.create_new_room(format!("Room #{}", idx), owner_id.clone());
        }
        let r = cm.create_new_room(format!("unable to create!"), owner_id.clone());

        assert!(r.is_err());
        assert_eq!(Error::TooManyRooms, r.err().unwrap());
    }

    #[test]
    fn cannot_use_room_name_twice() {
        let owner_id = String::from("user-a");
        let mut cm = ChatManager::new();
        let one = cm.create_new_room(String::from("Room"), owner_id.clone());
        let two = cm.create_new_room(String::from("Room"), owner_id.clone());

        assert!(one.is_ok());
        assert!(two.is_err());
        assert_eq!(Error::NameTaken, two.err().unwrap());
    }

    #[test]
    fn unused_room_name_is_available() {
        let owner_id = String::from("user-a");
        let mut cm = ChatManager::new();
        cm.create_new_room(String::from("Room 1"), owner_id.clone());
        cm.create_new_room(String::from("Room 2"), owner_id.clone());

        assert!(cm.name_is_available(&String::from("Room 3")));
    }

    #[test]
    #[should_panic]
    fn illegal_to_start_manager_twice() {
        let mut cm = ChatManager::new();
        cm.run(SocketAddr::new(IpAddr::from([127,0,0,1]), 8080));
        cm.run(SocketAddr::new(IpAddr::from([127,0,0,1]), 8080));
    }

    #[test]
    fn room_is_deleted_by_owner() {
        let owner_id = String::from("user-a");
        let mut cm = ChatManager::new();
        let name = String::from("Test Room");
        let room = cm.create_new_room(name.clone(), owner_id.clone()).unwrap();
        assert!(cm.list_rooms().contains(&name));
        let res = cm.delete_room(room.id, owner_id);
        assert!(res.is_ok());
        assert!(!cm.list_rooms().contains(&name));
    }

    #[test]
    fn room_cannot_be_deleted_by_non_owner() {
        let owner_id = String::from("user-a");
        let non_owner = String::from("user-b");
        let mut cm = ChatManager::new();
        let name = String::from("Test Room");
        let room = cm.create_new_room(name.clone(), owner_id.clone()).unwrap();
        assert!(cm.list_rooms().contains(&name));
        let res = cm.delete_room(room.id, non_owner);
        assert!(res.is_err());
        assert_eq!(Error::NotOwner, res.err().unwrap());
        assert!(cm.list_rooms().contains(&name));
    }

    #[test]
    fn cannot_delete_non_existant_room() {
        let owner_id = String::from("user-a");
        let mut cm = ChatManager::new();
        let name = String::from("Test Room");
        let room = cm.create_new_room(name.clone(), owner_id.clone());
        assert!(cm.list_rooms().contains(&name));
        let res = cm.delete_room(String::from("Test Room 2"), owner_id.clone());
        assert!(res.is_err());
        assert_eq!(Error::RoomNotFound, res.err().unwrap());
    }

    #[test]
    fn create_new_room_returns_name_and_id() {
        let owner_id = String::from("user-a");
        let mut cm = ChatManager::new();
        let name = String::from("test room");
        let result = cm.create_new_room(name.clone(), owner_id.clone());
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(name, data.name);
        assert!(data.id.len() > 0);
    }
}