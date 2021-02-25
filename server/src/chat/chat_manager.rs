use crate::chat::chat_room::{ChatRoom, Extractor, ChatData};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::net::{TcpListener, SocketAddr, TcpStream};
use threadpool::ThreadPool;
use std::sync::mpsc::Sender;
use std::io::{Write};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::borrow::Cow;
use log::info;
use crate::chat::chat_manager::Error::{TooManyRooms};
use std::fmt;

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
            started: AtomicBool::new(false)
        }
    }

    pub fn run_manager(&mut self, server_addr: SocketAddr) {
        if self.started.load(Ordering::Relaxed) {
            panic!("Illegal operation to start room twice")
        } else {
            let rooms_clone = self.rooms.clone();
            self.thread.lock().unwrap().execute(move || {
                let conn = TcpListener::bind(server_addr).unwrap();
                info!("Chat server is up and running... waiting for connections.");
                for new_stream in conn.incoming() {
                    let mut stream = new_stream.unwrap();
                    if let Some(name) = ChatManager::get_room_name(&mut stream) {
                        for (room, tx) in rooms_clone.lock().unwrap().iter() {
                            if room.name.eq(&name) {
                                return tx.send(stream).unwrap();
                            }
                        }
                        stream.write(b"HTTP/1.1 404 NOT FOUND");
                        // if let Some(found) = rooms_clone.lock().unwrap().g(&name) {
                        //     found.send(stream);
                        // } else {
                        //     stream.write(b"HTTP/1.1 404 NOT FOUND");
                        // }
                    }
                }
            });
            self.started.store(true, Ordering::Relaxed);
        }
    }

    //parse room name from raw HTTP headers
    fn get_room_name(new_stream: &mut TcpStream) -> Option<String> {
        let mut buff = [0; 1024];
        new_stream.peek(&mut buff);
        let incoming = String::from_utf8_lossy(&buff);
        if incoming.starts_with("GET /room/") {
            Some(ChatManager::extract_name(incoming))
        } else {
            None
        }
    }

    fn extract_name(http_req: Cow<str>) -> String {
        let idx = http_req.find("HTTP");
        let (right, _) = http_req.split_at(idx.unwrap());
        let name_start_idx = right.rfind("/").unwrap() + 1;
        let (_, name) =right.split_at(name_start_idx);
        String::from(name.trim())
    }

    pub fn create_new_room(&mut self, name: String) -> Result<(), Error> {
        if self.too_many_rooms() {
           Err(Error::TooManyRooms)
        } else {
            self.create_room(name)
        }
    }

    fn create_room(&mut self, name: String) -> Result<(), Error> {
        if self.name_is_unavailable(&name) {
            Err(Error::NameTaken)
        } else {
            let (room, client_rx) = mpsc::channel();
            let data = ChatData::new(name);
            self.rooms.lock().unwrap().insert(data.clone(), room);

            self.thread.lock().unwrap().execute(move || {
                let mut new_room = ChatRoom::new(data);
                new_room.run_room(client_rx);
            });
            Ok(())
        }
    }

    fn name_is_unavailable(&mut self, name: &String) -> bool {
        self.rooms.lock().unwrap().keys().any(|data| {
            data.name.to_lowercase() == name.to_lowercase()
        })
    }

    pub fn name_is_available(&self, name: &String) -> bool {
        !self.rooms.lock().unwrap().keys().any(|room|{
            room.name.to_lowercase() == name.to_lowercase()
        })
    }

    pub fn list_rooms(&self) -> Vec<String> {
        let mut vec = vec![];
        for room in self.rooms.lock().unwrap().keys() {
            vec.push(room.name.clone());
        }
        vec
    }

    pub fn get_room_data<T: Extractor>(&self, extractor: &mut T) {
        for (room, _) in self.rooms.lock().unwrap().iter() {
            room.extract_room_data(extractor);
        }
    }

    pub fn too_many_rooms(&mut self) -> bool {
        self.rooms.lock().unwrap().len() >= ROOM_LIMIT
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    TooManyRooms,
    RoomNotFound,
    NameTaken
}

impl std::error::Error for Error{}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Error::TooManyRooms => write!(f, "Too many rooms running"),
            Error::RoomNotFound => write!(f, "Room doesn't exist"),
            Error::NameTaken => write!(f, "Name is already in use"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::chat::chat_manager::ChatManager;
    use crate::chat::chat_manager::Error;


    #[test]
    fn can_parse_room_name() {
        let s = String::from_utf8_lossy(b"GET /room/hello HTTP/1.1\nHost: 127.0.0.1:8080");
        let name = ChatManager::extract_name(s);
        assert_eq!("hello", name);
    }

    #[test]
    fn can_create_up_to_ten_chat_rooms() {
        let mut cm = ChatManager::new();
        for idx in 0..10 {
            cm.create_new_room(format!("Room #{}", idx));
        }

        assert_eq!(10, cm.list_rooms().len())
    }

    #[test]
    fn creating_more_than_ten_rooms_causes_error() {
        let mut cm = ChatManager::new();
        for idx in 0..10 {
            cm.create_new_room(format!("Room #{}", idx));
        }
        let r = cm.create_new_room(format!("unable to create!"));

        assert!(r.is_err());
        assert_eq!(Error::TooManyRooms, r.err().unwrap());
    }

    #[test]
    fn cannot_use_room_name_twice() {
        let mut cm = ChatManager::new();
        let one = cm.create_new_room(String::from("Room"));
        let two = cm.create_new_room(String::from("Room"));

        assert!(one.is_ok());
        assert!(two.is_err());
        assert_eq!(Error::NameTaken, two.err().unwrap());
    }

    #[test]
    fn unused_room_name_is_available() {
        let mut cm = ChatManager::new();
        cm.create_new_room(String::from("Room 1"));
        cm.create_new_room(String::from("Room 2"));

        assert!(cm.name_is_available(&String::from("Room 3")));
    }
}