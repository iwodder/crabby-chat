use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Mutex};
use std::net::TcpStream;
use tungstenite::{WebSocket, Message};
use std::thread;

pub struct User {
    name: String,
    // user_rx: Mutex<Receiver<String>>,
    // user_tx: Sender<String>,
    // room: Sender<String>
}

impl User {
    pub fn new(name: String) -> Self {
        // let (tx, rx) = mpsc::<String>(channel);
        User {
            name,
            // user_rx: Mutex::new(user_rx),
            // user_tx: tx,
            // room
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    // pub fn user(&self) -> Sender<String> {
    //     self.user_tx.clone()
    // }

    pub fn run_user(&self, mut ws: WebSocket<TcpStream>, room: Sender<Message>, user_rx: Receiver<Message>) {
        let locked = Mutex::new(user_rx);
        unsafe {
            let output = thread::Builder::new().spawn_unchecked(|| {
                loop {
                    let msg = locked.lock().unwrap().recv().unwrap();
                    let result = ws.write_message(msg);
                    match result {
                        Ok(()) => (),
                        Err(err) => {
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
                    Err(err) => {
                        break;
                    }
                }
            }
            output.join();
        }
    }

    // pub async fn run_user(&self, mut ws: WebSocket<TcpStream>, room: Sender<Message>, user_rx: Receiver<Message>) {
    //
    // }
}
