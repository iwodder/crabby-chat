use crate::chat::chat_room::ChatRoom;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::{TcpListener, SocketAddr, TcpStream};
use threadpool::ThreadPool;
use std::sync::mpsc::Sender;
use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::borrow::Cow;
use log::info;

const ROOM_LIMIT: usize = 10;
const ROOM_LIMIT_AND_MGR: usize = ROOM_LIMIT+1;

pub struct ChatManager {
    rooms: Arc<Mutex<HashMap<String, Sender<TcpStream>>>>,
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
                        if let Some(found) = rooms_clone.lock().unwrap().get_mut(&name) {
                            found.send(stream);
                        } else {
                            stream.write(b"HTTP/1.1 404 NOT FOUND");
                        }
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
            Err(Error::TooManyRooms(String::from("Room limit exceeded.")))
        } else {
            let (room, client_rx) = mpsc::channel();
            self.rooms.lock().unwrap().insert(name.clone(), room);

            self.thread.lock().unwrap().execute(move || {
                let mut new_room = ChatRoom::new(name);
                new_room.run_room(client_rx);
            });
            Ok(())
        }
    }

    pub fn list_rooms(&self) -> Vec<String>{
        let mut vec = vec![];
        for room in self.rooms.lock().unwrap().keys() {
            vec.push(String::from(room));
        }
        vec
    }

    pub fn too_many_rooms(&mut self) -> bool {
        self.rooms.lock().unwrap().len() >= ROOM_LIMIT
    }
}

#[derive(Debug)]
pub enum Error {
    TooManyRooms(String),
    RoomNotFound(String)
}

#[cfg(test)]
mod test {
    use crate::chat::chat_manager::ChatManager;


    #[test]
    fn can_parse_room_name() {
        let s = String::from_utf8_lossy(b"GET /room/hello HTTP/1.1\nHost: 127.0.0.1:8080");
        let name = ChatManager::extract_name(s);
        assert_eq!("hello", name);
    }

}