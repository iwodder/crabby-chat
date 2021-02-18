use crate::chat::chat_room::ChatRoom;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::{TcpListener, SocketAddr, IpAddr, TcpStream};
use threadpool::ThreadPool;
use std::sync::mpsc::Sender;
use std::io::{Read, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::spawn;

const ROOM_LIMIT: usize = 10;

pub struct ChatManager {
    rooms: Arc<Mutex<HashMap<String, Sender<TcpStream>>>>,
    thread: ThreadPool
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            thread: ThreadPool::new(11)
        }
    }

    pub fn run_manager(&mut self, server_addr: SocketAddr) {
        let rooms_clone = self.rooms.clone();
        self.thread.execute(move || {
            let conn = TcpListener::bind(server_addr).unwrap();
            for new_stream in conn.incoming() {
                let mut stream = new_stream.unwrap();
                let name = ChatManager::get_room_name(&mut stream);
                if let Some(found) = rooms_clone.lock().unwrap().get_mut(&name) {
                    found.send(stream);
                } else {
                    stream.write(b"HTTP/1.1 404 NOT FOUND");
            }
        }});
    }

    //parse room name from raw HTTP headers
    fn get_room_name(new_stream: &mut TcpStream) -> String{
        let mut buff = [0; 1024];
        new_stream.peek(&mut buff);
        let incoming = String::from_utf8_lossy(&buff);
        String::from("My First Room")
    }

    pub fn create_new_room(&mut self, name: String) -> Result<(), Error> {
        if self.too_many_rooms() {
            Err(Error::TooManyRooms(String::from("Room limit exceeded.")))
        } else {
            let (room, client_rx) = mpsc::channel();
            self.rooms.lock().unwrap().insert(name.clone(), room);

            self.thread.execute(move || {
                let mut new_room = ChatRoom::new(name);
                new_room.run_room_chan(client_rx);
            });
            Ok(())
        }
    }

    pub fn too_many_rooms(&mut self) -> bool {
        self.rooms.lock().unwrap().len() >= ROOM_LIMIT
    }

}

#[derive(Debug)]
pub enum Error {
    TooManyRooms(String),
}