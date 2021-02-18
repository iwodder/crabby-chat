pub mod chat_manager;
mod chat_room;


pub mod chat_data {

    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    pub struct RoomCreated {
        msg: String,
        port: usize
    }
}