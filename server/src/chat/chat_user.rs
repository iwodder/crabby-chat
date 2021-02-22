use std::sync::mpsc::{Sender, Receiver};
use std::sync::Mutex;
use std::net::TcpStream;
use tungstenite::{WebSocket, Message};
use std::thread;

pub struct User {
    name: String,
}

impl User {
    pub fn new(name: String) -> Self {
        User {
            name
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn run_user(&self, mut ws: WebSocket<TcpStream>, room: Sender<Message>, user_rx: Receiver<Message>) {
        let locked = Mutex::new(user_rx);
        unsafe {
            let output = thread::Builder::new().spawn_unchecked(|| {
                loop {
                    let msg = locked.lock().unwrap().recv().unwrap();
                    let result = ws.write_message(msg);
                    match result {
                        Ok(()) => (),
                        Err(_) => {
                            break;
                        }
                    }
                    ws.write_pending();
                }
            }).unwrap();

            loop {
                match ws.read_message() {
                    Ok(msg) => {
                        room.send(msg);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            output.join();
        }
    }
}
